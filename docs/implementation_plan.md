# Factorio Mod 自動翻訳ツール - 実装計画

## 概要

Factorio 2.x (Space Age) 対応のMod翻訳ファイル(.cfg)を自動翻訳・管理するWindows向けWPF GUIアプリケーション「**FactorioModTranslator**」を開発する。MVVM パターンに基づくクリーンなアーキテクチャで構築する。

## 前提条件

> [!IMPORTANT]
> .NET 8 SDKが未インストールです。開発開始前にインストールが必要です。

## アーキテクチャ

```
FactorioModTranslator.sln
├── src/
│   └── FactorioModTranslator/          ← WPFアプリ本体
│       ├── Models/                      ← データモデル
│       ├── Services/                    ← ビジネスロジック
│       ├── ViewModels/                  ← VM (MVVM)
│       ├── Views/                       ← XAML画面
│       ├── Resources/                   ← 多言語リソース
│       ├── Converters/                  ← WPFコンバーター
│       └── Helpers/                     ← ユーティリティ
└── tests/
    └── FactorioModTranslator.Tests/     ← xUnitテスト
```

MVVM パターン: View → ViewModel → Service → Model

---

## 提案する変更内容

### フェーズ1: 環境構築

#### [NEW] .NET 8 SDK インストール
- wingetで.NET 8 SDKをインストール

#### [NEW] ソリューション・プロジェクト作成
- `dotnet new sln` でソリューション作成
- `dotnet new wpf` でWPFプロジェクト作成
- `dotnet new xunit` でテストプロジェクト作成

#### NuGet パッケージ
| パッケージ | 用途 |
|---|---|
| DeepL.net | DeepL翻訳API |
| Google.Cloud.Translation.V2 | Google翻訳API |
| Microsoft.Data.Sqlite | SQLite履歴DB |
| CommunityToolkit.Mvvm | MVVM基盤 (RelayCommand等) |
| System.IO.Compression | ZIP読み込み |

---

### フェーズ2: コア層（Models / Services）

#### [NEW] Models/CfgEntry.cs
- セクション名・キー・値を保持するデータモデル

#### [NEW] Models/ModInfo.cs
- Mod情報（名前、バージョン、パス、localeファイル一覧）

#### [NEW] Models/TranslationRecord.cs
- 翻訳履歴レコード

#### [NEW] Models/GlossaryEntry.cs
- 用語集エントリ

---

#### [NEW] Services/CfgParser.cs
- .cfgファイルの読み書き
- セクションヘッダー `[section-name]` の解析
- `key=value` 形式の解析、コメント行(`; ...`)の保持

#### [NEW] Services/ModLoader.cs
- ローカルフォルダおよびZIPファイルからMod読み込み
- `info.json` からMod情報取得
- `locale/` ディレクトリからcfgファイル収集

#### [NEW] Services/ITranslationEngine.cs (インターフェース)
- 翻訳メソッドの抽象化

#### [NEW] Services/DeepLTranslationEngine.cs
- DeepL.net SDKによる翻訳実装
- グロッサリー対応

#### [NEW] Services/GoogleTranslationEngine.cs
- Google.Cloud.Translation.V2による翻訳実装

#### [NEW] Services/VanillaTranslationService.cs
- Factorioインストールフォルダからバニラ翻訳データ取得
- キーベースマッチング → テキストベースマッチング → 参考渡し

#### [NEW] Services/GlossaryService.cs
- 用語集のCRUD操作（JSONファイル保存）

#### [NEW] Services/TranslationHistoryService.cs
- SQLiteによる翻訳履歴の保存・取得

#### [NEW] Services/SettingsService.cs
- アプリ設定のJSON保存/読み込み
- APIキーのDPAPI暗号化保存

#### [NEW] Services/LocalizationService.cs
- UI言語切替（日本語/英語）

---

### フェーズ3: ViewModel層

#### [NEW] ViewModels/MainViewModel.cs
- アプリ全体のナビゲーション管理
- 言語切替コマンド

#### [NEW] ViewModels/ModSelectionViewModel.cs
- Mod読み込み（フォルダ選択/ZIP選択）
- 翻訳モード選択（新規/差分/上書き/手動）
- 翻訳元・翻訳先言語選択

#### [NEW] ViewModels/TranslationPreviewViewModel.cs
- 翻訳結果のプレビュー表示
- セル編集による手動修正
- バニラ訳語のハイライト
- 保存コマンド

#### [NEW] ViewModels/SettingsViewModel.cs
- APIキー入力・保存
- 翻訳エンジン切替
- Factorioインストールパス設定

#### [NEW] ViewModels/GlossaryViewModel.cs
- 用語の追加・編集・削除

---

### フェーズ4: View層（WPF XAML）

#### [NEW] Views/MainWindow.xaml
- タブまたはナビゲーション形式のメインUI
- ヘッダーに言語切替ボタン配置

#### [NEW] Views/ModSelectionView.xaml
- フォルダ/ZIP選択ボタン
- Mod情報表示エリア
- 翻訳モード・言語選択コンボボックス
- 翻訳実行ボタン

#### [NEW] Views/TranslationPreviewView.xaml
- DataGridによる翻訳結果テーブル（セクション/キー/原文/翻訳/バニラ訳語）
- 編集可能なセル
- 保存・エクスポートボタン

#### [NEW] Views/SettingsView.xaml
- APIキー入力フィールド（マスク表示）
- 翻訳エンジン選択ラジオボタン
- Factorioパス選択

#### [NEW] Views/GlossaryView.xaml
- 用語一覧DataGrid
- 追加/編集/削除ボタン

#### デザイン方針
- ダークテーマベースのモダンUI
- Factorioのゲームカラーに合わせた配色（オレンジ/グレー系）
- アイコン付きTabControlによるナビゲーション

---

### フェーズ5: 仕上げ

#### [NEW] README.md
- プロジェクト概要、スクリーンショット、使い方、ビルド方法

#### [NEW] LICENSE
- MITライセンス

#### [NEW] .gitignore
- .NET用gitignore

---

## 検証計画

### 自動テスト (xUnit)

#### CfgParserTests
```bash
dotnet test tests/FactorioModTranslator.Tests --filter CfgParserTests
```
- cfgファイルの解析が正しいこと（セクション、key=value、コメント）
- cfgファイルの書き出しが元のフォーマットを保持すること

#### ModLoaderTests
```bash
dotnet test tests/FactorioModTranslator.Tests --filter ModLoaderTests
```
- フォルダからのMod読み込み
- ZIPからのMod読み込み

#### VanillaTranslationServiceTests
```bash
dotnet test tests/FactorioModTranslator.Tests --filter VanillaTranslationServiceTests
```
- キーベースマッチング
- テキストベースマッチング

#### ユースケースシナリオテスト (UseCaseTests)
```bash
dotnet test tests/FactorioModTranslator.Tests --filter UseCaseTests
```
- **UC1 (Modを読み込む)**
  - 正常系: ModLoaderを用いて指定フォルダ/ZIPから情報を読み込み、cfgエントリーが正しく抽出されること
  - 異常系: info.jsonなし、localeフォルダなし、cfg解析失敗、ZIP破損の各ケースで適切なエラーを返すこと
- **UC2 (翻訳を実行する)**
  - 正常系: TranslationOrchestrator経由でバニラ訳・用語集・履歴・APIモックの順に翻訳が適用されること
  - 異常系: APIキー未設定、タイムアウト、レート制限、無効なキー、キャンセル時のハンドリングが正しく行われること
- **UC3 (翻訳をプレビュー・編集する)**
  - 正常系: 翻訳結果のアイテムを編集し、状態が `Manual` に変更されること
  - 代替系: リバート（元に戻す）操作で変更前のテキストに戻り状態がリセットされること
- **UC4 (翻訳ファイルを保存する)**
  - 正常系: CfgParserを用いて、翻訳後データが正しいlocaleディレクトリ構造でファイル出力されること
  - 異常系: 書込権限なし、ディスク容量不足時のエラーハンドリングが正しく行われること
- **UC5 (設定を管理する)**
  - 正常系: SettingsServiceを用いて、APIキーや翻訳エンジンの設定が正しく保存・ロードされること
  - 異常系/代替系: テスト接続失敗、無効なFactorioパスの場合の警告表示が適切に機能すること
- **UC6 (用語集を管理する)**
  - 正常系: GlossaryServiceで用語の追加・編集が行え、翻訳時に正しく処理されること
  - 代替系: 重複登録時の確認、外部ファイルからのインポート/エクスポートが正しく機能すること
- **UC7 (翻訳履歴を参照する)**
  - 正常系: TranslationHistoryServiceでSQLiteデータベースに履歴が保存され、結果を正しく取得できること
  - 異常系/代替系: 履歴なし時、SQLiteデータベース破損時のハンドリングが機能すること
- **UC8 (バニラ訳語を適用する)**
  - 正常系: VanillaTranslationServiceが提供するバニラ辞書により、指定キー/テキストで訳語が適用されること
  - 異常系/代替系: Factorioインストールパス未設定、バニラデータ読み込み失敗時にエラーで停止せずスキップされること
- **UC9 (UI言語を切り替える)**
  - 正常系: LocalizationServiceで言語を切り替えた際、変更イベントが発火し状態が更新されること
  - 異常系: 言語リソースファイルが見つからない場合にデフォルト言語にフォールバックすること

### 手動検証（ユーザーによる確認）
1. アプリを起動し、設定画面でAPIキーを入力できることを確認
2. サンプルModフォルダを読み込み、cfgファイルが解析されることを確認
3. 翻訳を実行し、プレビュー画面に結果が表示されることを確認
4. プレビュー画面で翻訳を手動編集し、保存できることを確認

### ビルド確認
```bash
dotnet build src/FactorioModTranslator/FactorioModTranslator.csproj
dotnet publish src/FactorioModTranslator/FactorioModTranslator.csproj -c Release -r win-x64 --self-contained
```
