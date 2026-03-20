# Factorio Mod Translator

Factorio 2.x (Space Age) 対応のMod翻訳ファイルを自動翻訳・管理するWindows向けGUIツールです。

## 主な機能

- **Mod読み込み**: フォルダのModを直接読み込み
- **翻訳エンジン**: DeepL API に対応
- **差分翻訳**: 既存の翻訳ファイルを活かし、未翻訳部分のみを自動抽出して翻訳
- **翻訳履歴**: 過去に行った翻訳をSQLiteに保存し、再利用可能
- **ダークテーマ**: Factorioを彷彿とさせるオレンジ・グレー基調のUI

## 技術スタック

- Rust / Tauri 2.x
- Vanilla JavaScript / HTML / CSS (フロントエンド)
- reqwest (HTTP通信 - DeepL API)
- rusqlite (SQLite - 翻訳履歴)
- keyring (APIキー保存 - OS Credential Manager)
- tauri-plugin-log (ログ出力)


## 使い方

1. アプリを起動し、[Settings] タブで翻訳APIキーを設定します。
2. [Mod Selection] タブで翻訳したいModのフォルダを選択します。
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
- **NSISインストーラ**: `src-tauri/target/release/bundle/nsis/FactorioModTranslator_x.x.x_x64-setup.exe` (~3.5MB)

Windows 10以降ではWebView2が標準搭載されているため、追加のランタイムインストールは不要です。

## ログ出力

アプリケーションのログは以下の場所に出力されます:

- **出力先**: `%LOCALAPPDATA%\FactorioModTranslator\logs\log_yyyyMMdd.log`
- **フォーマット**: `[タイムスタンプ] [ログレベル] [ファイル名:行番号] メッセージ`

Rustバックエンドのログとフロントエンドのログが同一ファイルに統合されます。

## ライセンス

MIT License
