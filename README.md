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

- C# (.NET 8) / WPF
- CommunityToolkit.Mvvm (MVVM Pattern)
- DeepL.net / Google.Cloud.Translation.V2
- Microsoft.Data.Sqlite
- System.Security.Cryptography.ProtectedData (APIキー暗号化)

## 使い方

1. アプリを起動し、[Settings] タブで翻訳APIキーを設定します。
2. [Mod Selection] タブで翻訳したいModのフォルダまたはZIPを選択します。
3. 翻訳モード、ソース言語（通常はen）、ターゲット言語（通常はja）を選択し [Execute Translation] をクリックします。
4. [Preview] タブで翻訳結果を確認・修正し、[Save to Mod] で保存します。

## ビルド手順

### 開発環境
- .NET 8 SDK

### ビルド
```bash
dotnet build
```

### リリース用パッケージの作成（自己完結型シングルファイル）
以下のコマンドを実行することで、依存関係を含んだ1つの実行ファイル（win-x64用）が生成されます。
※ `PublishReadyToRun=false` は、WPFのシングルファイル配布における表示上の問題を回避するために重要です。

```bash
dotnet publish src/FactorioModTranslator/FactorioModTranslator.csproj -c Release -r win-x64 --self-contained true -p:PublishSingleFile=true -p:PublishReadyToRun=false -p:IncludeNativeLibrariesForSelfExtract=true
```

## ライセンス

MIT License
