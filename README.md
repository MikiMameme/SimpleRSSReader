# RSS Reader  

[![Rust](https://img.shields.io/badge/Rust-1.70%2B-black?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
[![Made with Gemini](https://img.shields.io/badge/Made%20with-Gemini-blue?logo=google-gemini&logoColor=white)](https://gemini.google.com/)
[![Author](https://img.shields.io/badge/Author-Miki%20Mame-lightgrey)](https://github.com/MikiMameme)  
![スクリーンショット](https://github.com/MikiMameme/SimpleRSSReader/blob/main/Screenshot.png)  
  
Rust と GUIライブラリ `egui` を使用して作成した、シンプルで軽量なデスクトップ RSS リーダーです。  
プログラミング学習の一環として開発した成果物です。  
バージョン2.0より、**VOICEVOX** と連携した記事読み上げ機能が搭載されました。  

### 🔊 音声読み上げ機能 (New in v2.0)
VOICEVOXのAPIを利用し、ニュース記事をキャラクターボイスで読み上げます。

* **再生コントロール**: 選択記事の読み上げ、フィルタリングされた記事の一括連続再生。
* **VOICEVOXランチャー**:
    * アプリ内からVOICEVOXの実行ファイルパス(`.exe`)を指定・記憶。
    * VOICEVOXが未起動の場合、アプリ内のボタンから直接起動可能。
* **カスタマイズ**: 話者（キャラクター）、スタイル（感情）、読み上げ速度の変更。
* **自動巡回**: 設定した間隔でバックグラウンド更新を行い、新着記事があれば音声でアナウンス。  

## 主な機能
- **フィード管理**: RSS フィードの追加、および削除が可能です。
- **削除確認ダイアログ**: 誤操作による削除を防ぐための確認ウィンドウを表示します。
- **時系列表示**: 登録した全フィードから記事を取得し、最新順に並べて表示します。
- **フィルタリング**: 特定のサイトの記事のみを絞り込んで閲覧できます。
- **検索機能**: タイトルだけでなく、記事の説明文からも検索するため、うろ覚えのキーワードでも記事を探せます。
- **既読管理システム**: 記事のタイトルをクリックすると、履歴を保存、次回以降の表示が自動的にグレーアウト（灰色）され視覚的にわかりやすいです。  
  （既読履歴は本体に保存、履歴の削除は`Read.json`を削除するだけ。プライバシーに配慮した設計になっています）  
- **日本語対応**: `Noto Sans JP` フォントを内蔵しており、環境を選ばず日本語を表示可能です。

## 使い方
### 起動方法
1. 配布されている`RSS_Reader_v2.0.zip`をダウンロードし、展開してください。
2. 展開した`RSS_Reader_v2.0`フォルダ内にある `RSS_Reader_v2.0.exe` を実行してください。
3. 初回起動時はフィードが登録されていません。左側のフィードの「＋追加」よりお好みのRSSフィードを登録してください。
4. 二回目以降は、同フォルダ内に作成される `feeds.json` から自動的にフィード情報を読み込みます。

### feeds.jsonの構造(フィード管理システム)
{  
  "feeds": [  
    {  
      "name": "登録した名前",  
      "url": "登録したURL"  
    }  
  ]  
}    

### Read.jsonの構造(既読管理システム)  
　[  
  "https://example.com/news/article_01.html",  
  "https://exampleblog.jp/posts/12345",  
  "https://exampletech-news.net/entry/rust-update"  
]  

### settings.jsonの構造(VOICEVOX連携管理システム)  
  ※話者をずんだもんに設定し、VOICEVOXのパスを指定した際の例  
 {  
  "speaker_name": "ずんだもん",  
  "style_id": 3,  
  "speed": 1.0,  
  "timer_enabled": false,  
  "timer_interval_min": 30,  
  "right_panel_width": 400.0,  
  "voicevox_path": "C:\\Users\\user\\AppData\\Local\\Programs\\VOICEVOX\\VOICEVOX.exe"   
}  
  
## 💻 動作環境  
  
### RSSリーダーとして使う場合  
* Windows 10 / 11 が正常に動作し、インターネット接続されたパソコン。  
  
### VOICEVOX読み上げ機能を使う場合  
* **VOICEVOXが快適に動作するスペック**  
    * NVIDIAやAMDなどの「独立GPU」が搭載されているPCを強く推奨します。CPU生成、CPU内蔵GPUでも動作しますが読み上げに時間がかかりうまく読み上げできないことがあります。  
* 別途 [VOICEVOX](https://voicevox.hiroshiba.jp/) のインストールが必要です。
### MSVCR110.dll がないため、プログラムを開始できません。プログラムを再インストールすると、この問題が解決する場合があります。と出た場合  
Microsoft Visual C++ 再頒布可能パッケージをインストールすると解決できる場合があります。  
[サポートされている最新の Visual C++ 再頒布可能パッケージのダウンロード](https://learn.microsoft.com/ja-jp/cpp/windows/latest-supported-vc-redist?view=msvc-170)
      
## 使用アセット・ライブラリなど  
言語: Rust  
IDE: RustRover  
GUI: egui  
Font: Noto Sans Japanese (SIL Open Font License 1.1)  

## 協力
本プログラムの作成にあたっては、生成AI（Gemini, Claude等）の協力を得て制作されました。  

## 免責事項
このソフトウェアは学習用に作成されました。  
このプログラムは細心の注意をもって作成されていますが、本プログラムを使用したことによって生じた損害等について、制作者（K.N）は一切の責任を負いません。利用者自身の責任においてご利用ください。  

## ライセンス
このプロジェクトのソースコードは MIT License の下で公開されています。  

## 更新履歴  

■2.0  
以下の機能を追加した。  
  
・VOICEVOXランチャー機能を実装  
　（アプリ内からの起動、パス設定、再接続機能の追加）  
・UIの微調整  
・VOICEVOX APIを利用した読み上げ機能の実装  
・話者選択、速度調整、感情設定機能の追加  
・自動巡回機能の追加  
  
■1.2  
以下の機能を追加した。
  
・既読管理システム（Read.json）  
記事のタイトルをクリックすると、その記事の「ユニークなURL」が保存、次回以降の表示が自動的にグレーアウト（灰色）され視覚的にわかりやすくした。  
read.json に履歴として保存、既読状態が維持される。  
  
・検索機能  
記事一覧の右上に検索バーを追加。  
絞り込み: 入力した文字が含まれる記事だけを表示。  
対象範囲: タイトルだけでなく、記事の説明文からも検索。  

■1.1  
並び順を「新→旧」にした  
  
■1.0  
公開  
  
  
  
