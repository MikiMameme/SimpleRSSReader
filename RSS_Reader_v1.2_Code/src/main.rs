//
// RSS Reader Ver_1.2
// Created by K.N (2026)
// Developed with the assistance of AI (Gemini, Claude)
// License: MIT
//

#![windows_subsystem = "windows"]

use chrono::{DateTime, Utc};
use eframe::egui;
use reqwest;
use rss::Channel;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

const FEEDS_FILE: &str = "feeds.json";
const READ_FILE: &str = "read.json";

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 700.0])
            .with_title("RSSリーダー"),
        ..Default::default()
    };

    eframe::run_native(
        "RSSリーダー",
        options,
        Box::new(|cc| {
            let mut fonts = egui::FontDefinitions::default();
            fonts.font_data.insert(
                "jp_font".to_owned(),
                egui::FontData::from_static(include_bytes!("../NotoSansJP-Regular.ttf")),
            );

            fonts.families.get_mut(&egui::FontFamily::Proportional)
                .unwrap()
                .insert(0, "jp_font".to_owned());

            cc.egui_ctx.set_fonts(fonts);

            Box::<RssReader>::default()
        }),
    )
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FeedSource {
    name: String,
    url: String,
}

#[derive(Debug, Clone)]
struct Article {
    title: String,
    description: String,
    link: String,
    pub_date: Option<DateTime<Utc>>,
    source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FeedsConfig {
    feeds: Vec<FeedSource>,
}

enum Screen {
    Main,
    AddFeed,
}

struct RssReader {
    screen: Screen,
    feeds: Vec<FeedSource>,
    articles: Vec<Article>,
    read_articles: HashSet<String>,

    selected_index: Option<usize>,
    selected_filter: Option<String>,
    search_query: String,

    descending: bool,

    new_feed_name: String,
    new_feed_url: String,
    confirm_delete_index: Option<usize>,
}

impl Default for RssReader {
    fn default() -> Self {
        let feeds = load_feeds();
        let read_articles = load_read_articles();

        let mut articles = if feeds.is_empty() {
            Vec::new()
        } else {
            fetch_all_feeds(&feeds)
        };

        let mut reader = Self {
            screen: Screen::Main,
            feeds,
            articles,
            read_articles,
            selected_index: None,
            selected_filter: None,
            search_query: String::new(),
            descending: true,
            new_feed_name: String::new(),
            new_feed_url: String::new(),
            confirm_delete_index: None,
        };

        reader.sort_articles();
        reader
    }
}

impl eframe::App for RssReader {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        match self.screen {
            Screen::Main => self.show_main_screen(ctx),
            Screen::AddFeed => self.show_add_feed_screen(ctx),
        }
        if let Some(index) = self.confirm_delete_index {
            egui::Window::new("確認")
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.label("本当にこのフィードを削除しますか？");
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui.button("削除").clicked(){
                            self.feeds.remove(index);
                            save_feeds(&self.feeds);
                            self.articles = fetch_all_feeds(&self.feeds);
                            self.sort_articles();
                            self.selected_filter = None;
                            self.confirm_delete_index = None;
                        }
                        if ui.button("キャンセル").clicked(){
                            self.confirm_delete_index = None;
                        }
                    });
                });
        }
    }
}

impl RssReader {
    fn sort_articles(&mut self) {
        let is_desc = self.descending;
        self.articles.sort_by(|a, b| {
            let cmp = match (&a.pub_date, &b.pub_date) {
                (Some(da), Some(db)) => da.cmp(db),
                (None, Some(_)) => std::cmp::Ordering::Less,
                (Some(_), None) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            };
            if is_desc { cmp.reverse() } else { cmp }
        });
    }

    fn show_main_screen(&mut self, ctx: &egui::Context) {
        if self.feeds.is_empty() {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(100.0);
                    ui.heading("RSSフィードが登録されていません");
                    ui.add_space(20.0);
                    ui.label("フィードを追加します");
                    ui.add_space(30.0);
                    if ui.button("フィードを追加").clicked(){
                        self.screen = Screen::AddFeed;
                    }
                });
            });
            return;
        }
        egui::SidePanel::left("feeds")
            .resizable(false)
            .min_width(150.0)
            .show(ctx, |ui| {
                ui.heading("フィード");
                ui.separator();
                if ui.selectable_label(self.selected_filter.is_none(),"すべて").clicked() {
                    self.selected_filter = None;
                    self.selected_index = None;
                }

                let mut feed_index_to_remove = None;

                for(i, feed) in self.feeds.iter().enumerate(){
                    ui.horizontal(|ui| {
                        let is_selected = self.selected_filter.as_ref() == Some(&feed.name);
                        if ui.selectable_label(is_selected, &feed.name).clicked() {
                            self.selected_filter = Some(feed.name.clone());
                            self.selected_index = None;
                        }
                        if ui.button("🗑️").clicked() {
                            self.confirm_delete_index = Some(i);
                        }
                    });
                }
                if let Some(index) = feed_index_to_remove {
                    self.feeds.remove(index);
                    save_feeds(&self.feeds);
                    self.articles = fetch_all_feeds(&self.feeds);
                    self.selected_filter = None;
                }

                ui.add_space(20.0);
                ui.separator();

                if ui.button("追加").clicked() {
                    self.screen = Screen::AddFeed;
                }
                if ui.button("更新").clicked() {
                    self.articles = fetch_all_feeds(&self.feeds);
                    self.sort_articles();
                    self.selected_index = None;
                }
                let sort_label = if self.descending { "並び順: 新着順" } else { "並び順: 古い順" };
                if ui.button(sort_label).clicked() {
                    self.descending = !self.descending;
                    self.sort_articles();
                }
            });

        egui::SidePanel::right("detail")
            .resizable(false)
            .min_width(400.0)
            .show(ctx, |ui| {
                ui.heading("詳細");
                ui.separator();

                if let Some(index) = self.selected_index {
                    if let Some(article) = self.articles.get(index) {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ui.add_space(10.0);
                            ui.label(egui::RichText::new(&article.title).size(18.0).strong());
                            ui.add_space(5.0);

                            let date_str = if let Some(dt) = article.pub_date {
                                dt.format("%Y/%m/%d %H:%M").to_string()
                            } else {
                                "日時不明".to_string()
                            };
                            ui.label(egui::RichText::new(format!("{} - {}", article.source, date_str)).size(12.0).weak());

                            ui.add_space(15.0);
                            ui.label(&article.description);
                            ui.add_space(15.0);
                            ui.horizontal(|ui| {
                                ui.label("リンク：");
                                ui.hyperlink_to(&article.link, &article.link);
                            });
                        });
                    }
                } else {
                    ui.vertical_centered(|ui| {
                        ui.add_space(50.0);
                        ui.label("記事を選択してください");
                    });
                }

            });
        egui::CentralPanel::default().show(ctx, |ui| {

            ui.horizontal(|ui| {
                ui.heading("記事一覧");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add(egui::TextEdit::singleline(&mut self.search_query).hint_text("🔍 検索..."));
                });
            });
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                for (index, article) in self.articles.iter().enumerate() {

                    if let Some(ref filter) = self.selected_filter {
                        if &article.source != filter {
                            continue;
                        }
                    }

                    if !self.search_query.is_empty() {
                        let query = self.search_query.to_lowercase();
                        if !article.title.to_lowercase().contains(&query) && !article.description.to_lowercase().contains(&query) {
                            continue;
                        }
                    }

                    let is_selected = self.selected_index == Some(index);
                    let is_read = self.read_articles.contains(&article.link);

                    ui.vertical(|ui| {

                        let title_text = if is_read {
                            egui::RichText::new(&article.title).color(egui::Color32::GRAY)
                        } else {
                            egui::RichText::new(&article.title).strong()
                        };

                        let title_label = egui::SelectableLabel::new(is_selected, title_text);
                        if ui.add(title_label).clicked() {
                            self.selected_index = Some(index);

                            if self.read_articles.insert(article.link.clone()) {
                                save_read_articles(&self.read_articles);
                            }
                        }

                        let date_str = if let Some(dt) = article.pub_date {
                            dt.format("%Y/%m/%d %H:%M").to_string()
                        } else {
                            "".to_string()
                        };
                        ui.label(egui::RichText::new(format!("{}  {}", article.source, date_str)).size(11.0).weak());
                    });

                    ui.separator();
                }
            });
        });
    }

    fn show_add_feed_screen(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                ui.heading("フィード追加");
                ui.add_space(30.0);

                ui.horizontal(|ui| {
                    ui.label("名前：");
                    ui.add_space(20.0);
                    ui.text_edit_singleline(&mut self.new_feed_name);
                });

                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.label("URL:");
                    ui.add_space(30.0);
                    ui.text_edit_singleline(&mut self.new_feed_url);
                });

                ui.horizontal(|ui| {
                    if ui.button("追加").clicked() {
                        if !self.new_feed_name.is_empty() && !self.new_feed_url.is_empty(){
                            let new_feed = FeedSource {
                                name: self.new_feed_name.clone(),
                                url: self.new_feed_url.clone(),
                            };
                            self.feeds.push(new_feed);
                            save_feeds(&self.feeds);
                            self.articles = fetch_all_feeds(&self.feeds);
                            self.sort_articles();
                            self.screen = Screen::Main;
                            self.new_feed_name.clear();
                            self.new_feed_url.clear();
                        }
                    }
                    if ui.button("キャンセル").clicked() {
                        self.screen = Screen::Main;
                        self.new_feed_name.clear();
                        self.new_feed_url.clear();
                    }
                });
            });
        });
    }
}

fn load_feeds() -> Vec<FeedSource> {
    if Path::new(FEEDS_FILE).exists(){
        if let Ok(content) = fs::read_to_string(FEEDS_FILE) {
            if let Ok(config) = serde_json::from_str::<FeedsConfig>(&content) {
                return config.feeds;
            }
        }
    }
    Vec::new()
}

fn save_feeds(feeds: &[FeedSource]) {
    let config = FeedsConfig {
        feeds: feeds.to_vec(),
    };
    if let Ok(json) = serde_json::to_string_pretty(&config) {
        let _ = fs::write(FEEDS_FILE, json);
    }
}

fn load_read_articles() -> HashSet<String> {
    if Path::new(READ_FILE).exists() {
        if let Ok(content) = fs::read_to_string(READ_FILE) {
            if let Ok(read_list) = serde_json::from_str::<HashSet<String>>(&content) {
                return read_list;
            }
        }
    }
    HashSet::new()
}

fn save_read_articles(read_list: &HashSet<String>) {
    if let Ok(json) = serde_json::to_string_pretty(read_list) {
        let _ = fs::write(READ_FILE, json);
    }
}

fn fetch_all_feeds(feeds: &[FeedSource]) -> Vec<Article> {
    let mut all_articles = Vec::new();

    for feed in feeds {
        if let Ok(response) = reqwest::blocking::get(&feed.url) {
            if let Ok(content) = response.bytes() {
                if let Ok(channel) = Channel::read_from(&content[..]) {
                    for item in channel.items().iter().take(50) {
                        let pub_date = item
                            .pub_date()
                            .and_then(|date| DateTime::parse_from_rfc2822(date).ok())
                            .map(|dt| dt.with_timezone(&Utc));

                        all_articles.push(Article {
                            title: item.title().unwrap_or("タイトルなし").to_string(),
                            description: item
                                .description()
                                .unwrap_or("説明なし")
                                .to_string(),
                            link: item.link().unwrap_or("リンクなし").to_string(),
                            pub_date,
                            source: feed.name.clone(),
                        });
                    }
                }
            }
        }
    }
    all_articles
}