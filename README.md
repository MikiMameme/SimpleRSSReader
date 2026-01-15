# RSS Reader  

[![Rust](https://img.shields.io/badge/Rust-1.70%2B-black?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
[![Made with Gemini](https://img.shields.io/badge/Made%20with-Gemini-blue?logo=google-gemini&logoColor=white)](https://gemini.google.com/)
[![Author](https://img.shields.io/badge/Author-Miki%20Mame-lightgrey)](https://github.com/MikiMameme)  
![スクリーンショット](https://github.com/MikiMameme/SimpleRSSReader/blob/main/Screenshot.png)  
  
Rust と GUIライブラリ `egui` を使用して作成した、シンプルで軽量なデスクトップ RSS リーダーです。  
プログラミング学習の一環として開発した成果物です。

## 主な機能
- **フィード管理**: RSS フィードの追加、および削除が可能です。
- **削除確認ダイアログ**: 誤操作による削除を防ぐための確認ウィンドウを表示します。
- **時系列表示**: 登録した全フィードから記事を取得し、最新順に並べて表示します。
- **フィルタリング**: 特定のサイトの記事のみを絞り込んで閲覧できます。
- **日本語対応**: `Noto Sans JP` フォントを内蔵しており、環境を選ばず日本語を表示可能です。

## 使い方
### 起動方法
1. 配布されている`RSS_Reader_v1.0.zip`をダウンロードし、展開してください。
2. 展開した`RSS_Reader_v1.0`フォルダ内にある `RSS_Reader_v1.0.exe` を実行してください。
3. 初回起動時はフィードが登録されていません。画面の指示に従い、お好みのRSSフィードURLを登録してください。
4. 二回目以降は、同フォルダ内に作成される `feeds.json` から自動的にフィード情報を読み込みます。

### feeds.jsonの構造
{  
  "feeds": [  
    {  
      "name": "登録した名前",  
      "url": "登録したURL"  
    }  
  ]  
}    
  
### ビルド方法（開発者向け）
ソースコードからビルドする場合は、以下の準備が必要です。  
1. Rust (Cargo) のインストール  
2. プロジェクトルートに `NotoSansJP-Regular.ttf` を配置  
3. 以下のコマンドを実行  
   ```bash  
   cargo run --release  
  
### プロジェクト構成  
src/main.rs: アプリケーションのメインロジック  
Cargo.toml: 依存ライブラリの設定  
NotoSansJP-Regular.ttf: 内蔵日本語フォント  

### 使用アセット・ライブラリなど  
言語: Rust  
IDE: RustRover  
GUI: egui  
Font: Noto Sans Japanese (SIL Open Font License 1.1)  

### 協力
本プログラムの作成にあたっては、生成AI（Gemini, Claude等）の協力を得て制作されました。  

### 免責事項
このプログラムは細心の注意をもって作成されていますが、本プログラムを使用したことによって生じた損害等について、制作者（K.N）は一切の責任を負いません。利用者自身の責任においてご利用ください。  

### ライセンス
このプロジェクトのソースコードは MIT License の下で公開されています。  
