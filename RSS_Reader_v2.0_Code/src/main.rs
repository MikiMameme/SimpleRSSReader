//
// RSS Reader 2.0 With VOICEVOX
// Created by K.N (2026)
// Developed with the assistance of AI (Gemini, Claude)
// License: MIT
//

#![windows_subsystem = "windows"]

use chrono::{DateTime, Local, Utc, Datelike};
use eframe::egui;
use reqwest;
use rss::Channel;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::io::Cursor;
use std::path::Path;
use std::process::Command;
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread;
use std::time::{Duration, Instant};
use urlencoding::encode;

const FEEDS_FILE: &str = "feeds.json";
const READ_FILE: &str = "read.json";
const SETTINGS_FILE: &str = "settings.json";
const VOICEVOX_URL: &str = "http://127.0.0.1:50021";

#[derive(Debug, Serialize, Deserialize, Clone)]
struct AppSettings {
    speaker_name: String,
    style_id: u32,
    speed: f64,
    timer_enabled: bool,
    timer_interval_min: u32,
    right_panel_width: f32,
    voicevox_path: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            speaker_name: "ずんだもん".to_string(),
            style_id: 3,
            speed: 1.0,
            timer_enabled: false,
            timer_interval_min: 30,
            right_panel_width: 400.0,
            voicevox_path: String::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct VoiceStyle { name: String, id: u32 }
#[derive(Debug, Clone, Deserialize, Serialize)]
struct VoiceSpeaker { name: String, styles: Vec<VoiceStyle> }
struct PlayConfig { speaker_id: u32, speed: f64 }
enum AudioCommand { Play(Vec<String>, PlayConfig), Stop }

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FeedSource { name: String, url: String }
#[derive(Debug, Clone)]
struct Article { title: String, description: String, link: String, pub_date: Option<DateTime<Utc>>, source: String }
#[derive(Deserialize, Serialize)] struct FeedsConfig { feeds: Vec<FeedSource> }

enum Screen { Main, AddFeed }

fn main() -> Result<(), eframe::Error> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || { audio_worker(rx); });

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1150.0, 800.0])
            .with_title("RSSリーダー Ver2.0"),
        ..Default::default()
    };

    eframe::run_native("RSS Reader", options, Box::new(|cc| {
        cc.egui_ctx.set_visuals(egui::Visuals::light());
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert("jp_font".to_owned(), egui::FontData::from_static(include_bytes!("../NotoSansJP-Regular.ttf")));
        fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap().insert(0, "jp_font".to_owned());
        cc.egui_ctx.set_fonts(fonts);
        Box::new(RssReader::new(tx))
    }))
}

fn audio_worker(receiver: Receiver<AudioCommand>) {
    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
    let sink = rodio::Sink::try_new(&stream_handle).unwrap();
    loop {
        if let Ok(command) = receiver.recv() {
            match command {
                AudioCommand::Play(playlist, config) => {
                    sink.stop();
                    for text in playlist {
                        match receiver.try_recv() {
                            Ok(AudioCommand::Stop) | Ok(AudioCommand::Play(_, _)) => { sink.stop(); break; }
                            Err(TryRecvError::Disconnected) => return,
                            _ => {}
                        }

                        if let Ok(data) = fetch_voicevox_audio(&text, config.speaker_id, config.speed) {
                            if let Ok(source) = rodio::Decoder::new(Cursor::new(data)) {
                                sink.append(source);
                                sink.sleep_until_end();
                            }
                        } else {

                            std::thread::sleep(Duration::from_millis(500));
                        }
                    }
                }
                AudioCommand::Stop => { sink.stop(); }
            }
        }
    }
}

fn fetch_voicevox_audio(text: &str, speaker_id: u32, speed: f64) -> anyhow::Result<Vec<u8>> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()?;

    let query_url = format!("{}/audio_query?text={}&speaker={}", VOICEVOX_URL, encode(text), speaker_id);
    let mut query_json: serde_json::Value = client.post(query_url).header("Content-Length", 0).send()?.json()?;
    if let Some(obj) = query_json.as_object_mut() {
        obj.insert("speedScale".to_string(), serde_json::json!(speed));
    }
    let res = client.post(format!("{}/synthesis?speaker={}", VOICEVOX_URL, speaker_id)).json(&query_json).send()?;
    Ok(res.bytes()?.to_vec())
}

fn format_date(dt: Option<DateTime<Utc>>) -> String {
    let Some(utc_dt) = dt else { return "不明".to_string() };
    let local_dt = utc_dt.with_timezone(&Local);
    let now = Local::now();

    if local_dt.date_naive() == now.date_naive() {
        local_dt.format("今日 %H:%M").to_string()
    } else if local_dt.date_naive() == now.date_naive().pred_opt().unwrap() {
        local_dt.format("昨日 %H:%M").to_string()
    } else {
        local_dt.format("%m/%d %H:%M").to_string()
    }
}

struct RssReader {
    screen: Screen,
    feeds: Vec<FeedSource>,
    articles: Vec<Article>,
    read_articles: HashSet<String>,
    selected_index: Option<usize>,
    selected_filter: Option<String>,
    search_query: String,
    audio_tx: Sender<AudioCommand>,
    available_speakers: Vec<VoiceSpeaker>,
    settings: AppSettings,
    selected_speaker_idx: usize,
    last_check_instant: Instant,
    next_check_in: Duration,
    new_feed_name: String,
    new_feed_url: String,
    is_voicevox_connected: bool,
    delete_confirm_feed: Option<usize>,
}

impl RssReader {
    fn new(audio_tx: Sender<AudioCommand>) -> Self {
        let feeds = load_feeds();
        let read_articles = load_read_articles();
        let articles = fetch_all_feeds(&feeds);
        let saved_settings: AppSettings = fs::read_to_string(SETTINGS_FILE)
            .ok().and_then(|c| serde_json::from_str(&c).ok()).unwrap_or_default();


        let (speakers, connected) = match fetch_speakers() {
            Ok(s) => (s, true),
            Err(_) => (vec![VoiceSpeaker {
                name: "未接続".to_string(),
                styles: vec![VoiceStyle { name: "標準".to_string(), id: 0 }],
            }], false)
        };

        let speaker_idx = speakers.iter().position(|s| s.name == saved_settings.speaker_name).unwrap_or(0);

        Self {
            screen: Screen::Main, feeds, articles, read_articles,
            selected_index: None, selected_filter: None, search_query: String::new(),
            audio_tx, available_speakers: speakers, settings: saved_settings,
            selected_speaker_idx: speaker_idx, last_check_instant: Instant::now(),
            next_check_in: Duration::from_secs(0), new_feed_name: String::new(), new_feed_url: String::new(),
            is_voicevox_connected: connected,
            delete_confirm_feed: None,
        }
    }


    fn refresh_speakers(&mut self) {
        if let Ok(speakers) = fetch_speakers() {
            self.available_speakers = speakers;
            self.is_voicevox_connected = true;

            if let Some(idx) = self.available_speakers.iter().position(|s| s.name == self.settings.speaker_name) {
                self.selected_speaker_idx = idx;
            } else {
                self.selected_speaker_idx = 0;
            }
        }
    }


    fn launch_voicevox(&mut self) {
        if self.settings.voicevox_path.is_empty() { return; }


        let path = self.settings.voicevox_path.clone();
        if let Err(e) = Command::new(&path).spawn() {
            eprintln!("VOICEVOX launch failed: {}", e);
        } else {

        }
    }

    fn get_filtered_indices(&self) -> Vec<usize> {
        self.articles.iter().enumerate().filter_map(|(idx, a)| {
            if let Some(ref f) = self.selected_filter { if &a.source != f { return None; } }
            if !self.search_query.is_empty() {
                let q = self.search_query.to_lowercase();
                if !a.title.to_lowercase().contains(&q) && !a.description.to_lowercase().contains(&q) { return None; }
            }
            Some(idx)
        }).collect()
    }

    fn save_settings(&self) {
        let mut to_save = self.settings.clone();

        if self.is_voicevox_connected && self.selected_speaker_idx < self.available_speakers.len() {
            to_save.speaker_name = self.available_speakers[self.selected_speaker_idx].name.clone();
        }
        if let Ok(json) = serde_json::to_string_pretty(&to_save) { let _ = fs::write(SETTINGS_FILE, json); }
    }

    fn send_play_command(&self, texts: Vec<String>, with_intro: bool) {
        if texts.is_empty() || !self.is_voicevox_connected { return; }
        let mut playlist = Vec::new();
        if with_intro {
            let name = &self.available_speakers[self.selected_speaker_idx].name;
            playlist.push(format!("ボイスボックスの {} がお伝えします。", name));
        }
        playlist.extend(texts);
        let _ = self.audio_tx.send(AudioCommand::Play(playlist, PlayConfig {
            speaker_id: self.settings.style_id,
            speed: self.settings.speed,
        }));
    }

    fn tick_timer(&mut self) {
        if !self.settings.timer_enabled { return; }
        let elapsed = self.last_check_instant.elapsed();
        let interval = Duration::from_secs(self.settings.timer_interval_min as u64 * 60);
        if elapsed >= interval {
            self.last_check_instant = Instant::now();
            self.perform_auto_update();
        }
        self.next_check_in = interval.saturating_sub(elapsed);
    }

    fn perform_auto_update(&mut self) {
        let new_fetched = fetch_all_feeds(&self.feeds);
        let mut new_items = Vec::new();
        for art in &new_fetched {
            if !self.read_articles.contains(&art.link) { new_items.push(art.clone()); }
        }
        if !new_items.is_empty() {
            let count = new_items.len();
            let mut speech = vec![format!("定期巡回を完了しました。新着ニュースが {} 件あります。", count)];
            for art in new_items.iter().take(3) { speech.push(format!("タイトル。{}。", art.title)); }
            self.send_play_command(speech, true);
        }
        self.articles = new_fetched;
    }

    fn show_main_screen(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("left").resizable(false).min_width(240.0).show(ctx, |ui| {
            ui.add_space(5.0);
            ui.heading("📢 音声設定");
            ui.separator();


            ui.horizontal(|ui| {
                if self.is_voicevox_connected {
                    ui.label(egui::RichText::new("● 接続完了").color(egui::Color32::GREEN));
                } else {
                    ui.label(egui::RichText::new("● 未接続").color(egui::Color32::RED));
                    if ui.button("再接続").clicked() {
                        self.refresh_speakers();
                    }
                }
            });
            ui.add_space(5.0);


            ui.collapsing("VOICEVOX起動設定", |ui| {
                ui.label("実行ファイルのパス(.exe):");
                ui.horizontal(|ui| {

                    let path_display = if self.settings.voicevox_path.len() > 20 {
                        format!("...{}", &self.settings.voicevox_path[self.settings.voicevox_path.len()-20..])
                    } else {
                        self.settings.voicevox_path.clone()
                    };
                    ui.label(path_display);

                    if ui.button("📂").clicked() {

                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("exe", &["exe"])
                            .pick_file() {
                            self.settings.voicevox_path = path.to_string_lossy().to_string();
                            self.save_settings();
                        }
                    }
                });

                if !self.is_voicevox_connected {
                    if ui.button("🚀 アプリから起動する").clicked() {
                        self.launch_voicevox();

                    }
                }
            });
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("話者:");
                egui::ComboBox::from_id_source("sp")
                    .selected_text(&self.available_speakers[self.selected_speaker_idx].name)
                    .show_ui(ui, |ui| {
                        for (i, s) in self.available_speakers.iter().enumerate() {
                            if ui.selectable_value(&mut self.selected_speaker_idx, i, &s.name).clicked() {
                                if let Some(st) = s.styles.first() { self.settings.style_id = st.id; }
                                self.save_settings();
                            }
                        }
                    });
            });
            let styles = self.available_speakers[self.selected_speaker_idx].styles.clone();
            ui.horizontal(|ui| {
                ui.label("感情:");
                let cur = styles.iter().find(|s| s.id == self.settings.style_id).map(|s| s.name.as_str()).unwrap_or("");
                egui::ComboBox::from_id_source("st").selected_text(cur).show_ui(ui, |ui| {
                    for st in styles { if ui.selectable_value(&mut self.settings.style_id, st.id, &st.name).clicked() { self.save_settings(); } }
                });
            });
            ui.horizontal(|ui| {
                ui.label("速度:");
                if ui.add(egui::Slider::new(&mut self.settings.speed, 0.5..=2.0).step_by(0.1)).changed() { self.save_settings(); }
            });
            ui.add_space(10.0);
            ui.heading("⏱ ニュース巡回");
            ui.separator();
            if ui.checkbox(&mut self.settings.timer_enabled, "自動巡回を有効にする").changed() {
                self.last_check_instant = Instant::now();
                self.save_settings();
            }
            ui.horizontal(|ui| {
                ui.label("間隔:");
                if ui.add(egui::Slider::new(&mut self.settings.timer_interval_min, 1..=120).suffix("分")).changed() { self.save_settings(); }
            });
            if self.settings.timer_enabled {
                ui.label(egui::RichText::new(format!("次回の巡回まで: {}分{}秒", self.next_check_in.as_secs() / 60, self.next_check_in.as_secs() % 60)).color(egui::Color32::DARK_BLUE).size(12.0));
            }
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if ui.button("▶ 一括再生").clicked() {
                    let indices = self.get_filtered_indices();
                    let list = indices.iter().take(10).map(|&i| format!("{}。{}", self.articles[i].title, self.articles[i].description)).collect();
                    self.send_play_command(list, true);
                }
                if ui.button("⏹ 停止").clicked() { let _ = self.audio_tx.send(AudioCommand::Stop); }
            });
            ui.add_space(20.0);
            ui.heading("フィード");
            if ui.selectable_label(self.selected_filter.is_none(), "すべて").clicked() { self.selected_filter = None; }

            for i in 0..self.feeds.len() {
                let name = &self.feeds[i].name;
                ui.horizontal(|ui| {
                    if ui.selectable_label(self.selected_filter.as_ref() == Some(name), name).clicked() {
                        self.selected_filter = Some(name.clone());
                    }
                    if ui.button("🗑️").clicked() {
                        self.delete_confirm_feed = Some(i);
                    }
                });
            }
            if ui.button("＋ 追加").clicked() { self.screen = Screen::AddFeed; }
        });

        let right_res = egui::SidePanel::right("detail")
            .resizable(true)
            .default_width(self.settings.right_panel_width)
            .width_range(250.0..=800.0)
            .show(ctx, |ui| {
                if let Some(idx) = self.selected_index {
                    if let Some(a) = self.articles.get(idx).cloned() {
                        ui.heading(&a.title);
                        ui.add_space(5.0);
                        ui.horizontal(|ui| {
                            if ui.button("🔊 読む").clicked() {
                                self.send_play_command(vec![format!("{}。{}", a.title, a.description)], true);
                            }
                            if ui.button("🌐 ブラウザで開く").clicked() {
                                ui.ctx().output_mut(|o| o.open_url = Some(egui::OpenUrl::new_tab(&a.link)));
                            }
                        });
                        ui.separator();
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ui.label(egui::RichText::new(&a.description).size(14.0));
                        });
                    }
                } else {
                    ui.centered_and_justified(|ui| {
                        ui.label("記事を選択してください");
                    });
                }
            });

        let current_width = right_res.response.rect.width();
        if (current_width - self.settings.right_panel_width).abs() > 1.0 {
            self.settings.right_panel_width = current_width;
            self.save_settings();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("記事一覧");
                ui.add_space(20.0);
                ui.add(egui::TextEdit::singleline(&mut self.search_query).hint_text("検索..."));
            });
            ui.separator();
            egui::ScrollArea::vertical().show(ui, |ui| {
                let filtered_indices = self.get_filtered_indices();
                for idx in filtered_indices {
                    let a = &self.articles[idx];
                    let is_read = self.read_articles.contains(&a.link);
                    let text_color = if is_read { egui::Color32::GRAY } else { egui::Color32::BLACK };
                    let text = egui::RichText::new(&a.title).color(text_color).size(14.0);
                    if ui.selectable_label(self.selected_index == Some(idx), text).clicked() {
                        self.selected_index = Some(idx);
                        if self.read_articles.insert(a.link.clone()) { save_read_articles(&self.read_articles); }
                    }

                    ui.label(egui::RichText::new(format!("{} - {}", a.source, format_date(a.pub_date))).size(10.0).weak());
                }
            });
        });


        if let Some(feed_idx) = self.delete_confirm_feed {
            egui::Window::new("削除確認")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label(format!("「{}」を本当に削除しますか?", self.feeds[feed_idx].name));
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui.button("削除").clicked() {
                            let deleted_name = self.feeds[feed_idx].name.clone();
                            self.feeds.remove(feed_idx);
                            save_feeds(&self.feeds);
                            self.articles = fetch_all_feeds(&self.feeds);

                            if self.selected_filter.as_ref() == Some(&deleted_name) {
                                self.selected_filter = None;
                            }
                            self.delete_confirm_feed = None;
                        }
                        if ui.button("キャンセル").clicked() {
                            self.delete_confirm_feed = None;
                        }
                    });
                });
        }
    }

    fn show_add_feed_screen(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                ui.heading("フィード追加");
                ui.add(egui::TextEdit::singleline(&mut self.new_feed_name).hint_text("名前"));
                ui.add(egui::TextEdit::singleline(&mut self.new_feed_url).hint_text("URL"));
                ui.horizontal(|ui| {
                    if ui.button("追加").clicked() {
                        self.feeds.push(FeedSource { name: self.new_feed_name.clone(), url: self.new_feed_url.clone() });
                        save_feeds(&self.feeds);
                        self.articles = fetch_all_feeds(&self.feeds);
                        self.screen = Screen::Main;
                    }
                    if ui.button("キャンセル").clicked() { self.screen = Screen::Main; }
                });
            });
        });
    }
}

impl eframe::App for RssReader {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.tick_timer();
        if self.settings.timer_enabled { ctx.request_repaint_after(Duration::from_secs(1)); }
        match self.screen {
            Screen::Main => self.show_main_screen(ctx),
            Screen::AddFeed => self.show_add_feed_screen(ctx),
        }
    }
}


fn fetch_speakers() -> anyhow::Result<Vec<VoiceSpeaker>> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(1))
        .build()?;
    Ok(client.get(format!("{}/speakers", VOICEVOX_URL)).send()?.json()?)
}
fn load_feeds() -> Vec<FeedSource> { fs::read_to_string(FEEDS_FILE).ok().and_then(|c| serde_json::from_str::<FeedsConfig>(&c).ok()).map(|cfg| cfg.feeds).unwrap_or_default() }
fn save_feeds(f: &[FeedSource]) { let _ = fs::write(FEEDS_FILE, serde_json::to_string_pretty(&FeedsConfig { feeds: f.to_vec() }).unwrap_or_default()); }
fn save_read_articles(r: &HashSet<String>) { let _ = fs::write(READ_FILE, serde_json::to_string_pretty(r).unwrap()); }
fn load_read_articles() -> HashSet<String> { fs::read_to_string(READ_FILE).ok().and_then(|c| serde_json::from_str(&c).ok()).unwrap_or_default() }
fn fetch_all_feeds(feeds: &[FeedSource]) -> Vec<Article> {
    let mut articles = Vec::new();
    let client = reqwest::blocking::Client::builder().timeout(Duration::from_secs(3)).build().unwrap();
    for f in feeds {
        if let Ok(res) = client.get(&f.url).send() {
            if let Ok(b) = res.bytes() {
                if let Ok(ch) = Channel::read_from(&b[..]) {
                    for item in ch.items().iter().take(10) {
                        articles.push(Article {
                            title: item.title().unwrap_or("No Title").to_string(),
                            description: item.description().unwrap_or("").replace(char::is_whitespace, " ").chars().take(300).collect(),
                            link: item.link().unwrap_or("").to_string(),
                            pub_date: item.pub_date().and_then(|d| DateTime::parse_from_rfc2822(d).ok()).map(|dt| dt.with_timezone(&Utc)),
                            source: f.name.clone(),
                        });
                    }
                }
            }
        }
    }
    articles.sort_by(|a, b| b.pub_date.cmp(&a.pub_date));
    articles
}