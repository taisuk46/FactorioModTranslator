# Factorio Mod Translator

Factorio 2.x (Space Age) 対応のMod翻訳ファイルを自動翻訳・管理するWindows向けGUIツールです。

## 主な機能

- **Mod読み込み**: フォルダまたはZIP形式のModを直接読み込み
- **翻訳エンジン**: DeepL API / Google Translate API に対応
- **バニラ連携**: Factorio本体の公式訳語を優先的に適用し、用語の一貫性を保持
- **差分翻訳**: 既存の翻訳ファイルを活かし、未翻訳部分のみを自動抽出して翻訳
- **用語集管理**: 特定の用語の固定訳や翻訳除外設定が可能
- **翻訳履歴**: 過去に行った翻訳をSQLiteに保存し、再利用可能
- **ダークテーマ**: Factorioを彷彿とさせるオレンジ・グレー基調のUI

## 技術スタック

### Tauri版 (現行)
- Rust / Tauri 2.x
- Vanilla JavaScript / HTML / CSS (フロントエンド)
- reqwest (HTTP通信 - DeepL / Google Translate API)
- rusqlite (SQLite - 翻訳履歴)
- keyring (APIキー保存 - OS Credential Manager)
- tauri-plugin-log (ログ出力)

### WPF版 (参考保持: `src-wpf/`)
- C# (.NET 8) / WPF
- CommunityToolkit.Mvvm (MVVM Pattern)
- DeepL.net / Google.Cloud.Translation.V2
- Microsoft.Data.Sqlite
- System.Security.Cryptography.ProtectedData (APIキー暗号化)

## 使い方

1. アプリを起動し、[Settings] タブで翻訳APIキーを設定します。
2. [Mod Selection] タブで翻訳したいModのフォルダまたはZIPのパスを入力します。
3. 翻訳実行ボタンをクリックします。
4. [Preview] タブで翻訳結果を確認・修正し、保存します。

## ビルド手順

### 前提条件
- [Node.js](https://nodejs.org/) (LTS)
- [Rust](https://www.rust-lang.org/tools/install)
- [Tauri 2 CLI prerequisites](https://v2.tauri.app/start/prerequisites/)

### セットアップ
```bash
npm install
```

### 開発ビルド（デバッグ実行）
開発モードでアプリケーションを起動します。ホットリロードが有効です。

```bash
npm run tauri dev
```

### リリースビルド
最適化された実行ファイルとNSISインストーラを生成します。

```bash
npm run tauri build
```

### 生成物の場所
- **実行ファイル**: `src-tauri/target/release/factoriomodtranslator.exe`
- **NSISインストーラ**: `src-tauri/target/release/bundle/nsis/FactorioModTranslator_0.1.0_x64-setup.exe` (~3.5MB)

Windows 10以降ではWebView2が標準搭載されているため、追加のランタイムインストールは不要です。

## ログ出力

アプリケーションのログは以下の場所に出力されます:

- **出力先**: `%LOCALAPPDATA%\FactorioModTranslator\logs\log_yyyyMMdd.log`
- **フォーマット**: `[タイムスタンプ] [ログレベル] [ファイル名:行番号] メッセージ`

Rustバックエンドのログとフロントエンドのログが同一ファイルに統合されます。

## ディレクトリ構成

```
factoriomodtranslator_app/
├── src/                    # フロントエンド (HTML/CSS/JS)
│   ├── index.html          # メイン画面
│   ├── index.css           # デザインシステム
│   ├── styles.css          # コンポーネントスタイル
│   ├── main.js             # アプリロジック
│   └── locales.json        # UI文字列リソース (ja/en)
├── src-tauri/              # バックエンド (Rust)
│   ├── Cargo.toml          # Rust依存関係
│   ├── tauri.conf.json     # Tauri設定
│   └── src/
│       ├── main.rs         # エントリーポイント
│       ├── lib.rs          # アプリ初期化・プラグイン設定
│       ├── commands/       # IPCコマンド定義
│       ├── models/         # データモデル
│       └── services/       # ビジネスロジック
├── src-wpf/                # WPF版ソース (参考用保持)
├── docs/                   # 設計ドキュメント
│   ├── design_document.md      # WPF版設計資料
│   └── design_document_tauri.md # Tauri版設計資料
├── package.json
└── README.md
```

## 設計ドキュメント

- [Tauri版 設計資料](docs/design_document_tauri.md)
- [WPF版 設計資料](docs/design_document.md)

## ライセンス

MIT License
