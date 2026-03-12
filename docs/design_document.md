# FactorioModTranslator 設計資料

## 1. システム概要

Factorio 2.x (Space Age) 対応Modの翻訳ファイル(.cfg)を自動翻訳・管理するWindows GUIアプリケーション。

| 項目 | 内容 |
|---|---|
| アプリ名 | FactorioModTranslator |
| 対象OS | Windows 10/11 |
| フレームワーク | .NET 8 / WPF |
| アーキテクチャ | MVVM (Model-View-ViewModel) |
| 翻訳エンジン | DeepL API / Google Translate API |
| データ永続化 | SQLite (履歴) / JSON (設定・用語集) |
| 配布形態 | self-contained exe (ZIP) |
| ライセンス | MIT |

---

## 2. ユースケース図

```mermaid
graph LR
    User(("👤 ユーザー"))

    subgraph FactorioModTranslator
        UC1["Modを読み込む"]
        UC2["翻訳を実行する"]
        UC3["翻訳をプレビュー・編集する"]
        UC4["翻訳ファイルを保存する"]
        UC5["設定を管理する"]
        UC6["用語集を管理する"]
        UC7["翻訳履歴を参照する"]
        UC8["バニラ訳語を適用する"]
        UC9["UI言語を切り替える"]
    end

    subgraph External["外部システム"]
        DeepL["DeepL API"]
        Google["Google Translate API"]
        FS["ファイルシステム"]
    end

    User --> UC1
    User --> UC2
    User --> UC3
    User --> UC4
    User --> UC5
    User --> UC6
    User --> UC7
    User --> UC9

    UC1 --> FS
    UC2 --> DeepL
    UC2 --> Google
    UC2 -.->|include| UC8
    UC4 --> FS
    UC8 --> FS
```

### ユースケース一覧

| ID | ユースケース | 概要 | アクター |
|---|---|---|---|
| UC1 | Modを読み込む | ローカルフォルダまたはZIPファイルからModを読み込み、locale内のcfgファイルを解析 | ユーザー |
| UC2 | 翻訳を実行する | 選択した翻訳モード（新規/差分/上書き）で翻訳APIを呼び出し | ユーザー, 翻訳API |
| UC3 | 翻訳をプレビュー・編集する | 翻訳結果を表示し、手動で修正可能 | ユーザー |
| UC4 | 翻訳ファイルを保存する | cfgファイルとしてlocale構造でエクスポート | ユーザー |
| UC5 | 設定を管理する | APIキー、翻訳エンジン選択、Factorioパスを設定 | ユーザー |
| UC6 | 用語集を管理する | 固有名詞と固定訳の登録・編集・削除 | ユーザー |
| UC7 | 翻訳履歴を参照する | 過去の翻訳結果の閲覧、差分更新への再利用 | ユーザー |
| UC8 | バニラ訳語を適用する | Factorio公式訳語とマッチングし適用 | (UC2から呼出) |
| UC9 | UI言語を切り替える | 日本語/英語の表示切替 | ユーザー |

---

## 3. クラス図

### 3.1 全体構成

```mermaid
classDiagram
    direction TB

    namespace Models {
        class CfgEntry {
            +string Section
            +string Key
            +string Value
            +string Comment
        }
        class CfgFile {
            +string FilePath
            +string LanguageCode
            +List~CfgEntry~ Entries
            +List~string~ SectionOrder
        }
        class ModInfo {
            +string Name
            +string Version
            +string Title
            +string Author
            +string SourcePath
            +ModSourceType SourceType
            +List~CfgFile~ LocaleFiles
        }
        class TranslationRecord {
            +int Id
            +string ModName
            +string Section
            +string Key
            +string SourceLang
            +string TargetLang
            +string SourceText
            +string TranslatedText
            +string Engine
            +DateTime TranslatedAt
        }
        class GlossaryEntry {
            +string SourceTerm
            +string TargetTerm
            +string SourceLang
            +string TargetLang
            +bool ExcludeFromTranslation
        }
        class TranslationItem {
            +string Section
            +string Key
            +string SourceText
            +string TranslatedText
            +string VanillaTranslation
            +TranslationSource Source
            +bool IsEdited
        }
        class AppSettings {
            +TranslationEngineType SelectedEngine
            +string FactorioInstallPath
            +string UILanguage
            +string LastModPath
        }
    }

    namespace Enums {
        class ModSourceType {
            <<enumeration>>
            Folder
            Zip
        }
        class TranslationMode {
            <<enumeration>>
            NewTranslation
            DiffTranslation
            OverwriteUpdate
            ManualEdit
        }
        class TranslationEngineType {
            <<enumeration>>
            DeepL
            GoogleTranslate
        }
        class TranslationSource {
            <<enumeration>>
            API
            VanillaKeyMatch
            VanillaTextMatch
            Manual
            History
            Glossary
        }
    }

    CfgFile --> CfgEntry
    ModInfo --> CfgFile
    ModInfo --> ModSourceType
    TranslationItem --> TranslationSource
    AppSettings --> TranslationEngineType
```

### 3.2 サービス層

```mermaid
classDiagram
    direction TB

    class ITranslationEngine {
        <<interface>>
        +TranslateAsync(text, sourceLang, targetLang) Task~string~
        +TranslateBatchAsync(texts, sourceLang, targetLang) Task~List~string~~
        +GetSupportedLanguagesAsync() Task~List~LanguageInfo~~
        +SetGlossary(entries) void
    }

    class DeepLTranslationEngine {
        -DeepLClient _client
        -string _apiKey
        +TranslateAsync()
        +TranslateBatchAsync()
        +GetSupportedLanguagesAsync()
        +SetGlossary()
    }

    class GoogleTranslationEngine {
        -TranslationClient _client
        -string _apiKey
        +TranslateAsync()
        +TranslateBatchAsync()
        +GetSupportedLanguagesAsync()
        +SetGlossary()
    }

    class CfgParser {
        +Parse(stream) CfgFile
        +Write(cfgFile, stream) void
    }

    class ModLoader {
        +LoadFromFolder(path) ModInfo
        +LoadFromZip(path) ModInfo
        -ParseInfoJson(json) ModInfo
        -CollectLocaleFiles(basePath) List~CfgFile~
    }

    class VanillaTranslationService {
        -Dictionary _vanillaData
        +LoadVanillaData(factorioPath, lang) void
        +MatchByKey(section, key) string?
        +MatchByText(sourceText) string?
        +GetContextHints(sourceText) List~string~
    }

    class TranslationOrchestrator {
        -ITranslationEngine _engine
        -VanillaTranslationService _vanilla
        -GlossaryService _glossary
        -TranslationHistoryService _history
        +ExecuteTranslation(mod, mode, srcLang, tgtLang) Task~List~TranslationItem~~
    }

    class GlossaryService {
        -List~GlossaryEntry~ _entries
        +Load() void
        +Save() void
        +Add(entry) void
        +Remove(sourceTerm) void
        +GetAll() List~GlossaryEntry~
        +ApplyGlossary(text) string
    }

    class TranslationHistoryService {
        -SqliteConnection _db
        +SaveRecord(record) void
        +GetHistory(modName, key) List~TranslationRecord~
        +GetPreviousTranslation(modName, section, key, targetLang) string?
    }

    class SettingsService {
        +Load() AppSettings
        +Save(settings) void
        +SaveApiKey(engine, key) void
        +LoadApiKey(engine) string?
    }

    class LocalizationService {
        +CurrentLanguage string
        +SetLanguage(lang) void
        +GetString(key) string
        +LanguageChanged event
    }

    ITranslationEngine <|.. DeepLTranslationEngine
    ITranslationEngine <|.. GoogleTranslationEngine

    TranslationOrchestrator --> ITranslationEngine
    TranslationOrchestrator --> VanillaTranslationService
    TranslationOrchestrator --> GlossaryService
    TranslationOrchestrator --> TranslationHistoryService

    ModLoader --> CfgParser
```

### 3.3 ViewModel層

```mermaid
classDiagram
    direction TB

    class MainViewModel {
        +ObservableCollection~object~ Tabs
        +object SelectedTab
        +string CurrentLanguage
        +ICommand SwitchLanguageCommand
    }

    class ModSelectionViewModel {
        +ModInfo LoadedMod
        +TranslationMode SelectedMode
        +string SourceLanguage
        +string TargetLanguage
        +List~string~ AvailableLanguages
        +bool IsTranslating
        +double Progress
        +ICommand SelectFolderCommand
        +ICommand SelectZipCommand
        +ICommand StartTranslationCommand
    }

    class TranslationPreviewViewModel {
        +ObservableCollection~TranslationItem~ Items
        +TranslationItem SelectedItem
        +string FilterText
        +ICommand SaveCommand
        +ICommand ExportCommand
        +ICommand RevertCommand
    }

    class SettingsViewModel {
        +TranslationEngineType SelectedEngine
        +string DeepLApiKey
        +string GoogleApiKey
        +string FactorioPath
        +ICommand SaveCommand
        +ICommand BrowseFactorioPathCommand
        +ICommand TestApiKeyCommand
    }

    class GlossaryViewModel {
        +ObservableCollection~GlossaryEntry~ Entries
        +GlossaryEntry SelectedEntry
        +string NewSourceTerm
        +string NewTargetTerm
        +ICommand AddCommand
        +ICommand EditCommand
        +ICommand DeleteCommand
        +ICommand ImportCommand
        +ICommand ExportCommand
    }

    MainViewModel --> ModSelectionViewModel
    MainViewModel --> TranslationPreviewViewModel
    MainViewModel --> SettingsViewModel
    MainViewModel --> GlossaryViewModel
```

---

## 4. シーケンス図

### 4.1 Mod読み込み → 翻訳実行フロー

```mermaid
sequenceDiagram
    actor User as ユーザー
    participant View as ModSelectionView
    participant VM as ModSelectionVM
    participant ML as ModLoader
    participant Parser as CfgParser
    participant Orch as TranslationOrchestrator
    participant Vanilla as VanillaTranslationService
    participant Glossary as GlossaryService
    participant Engine as ITranslationEngine
    participant History as TranslationHistoryService
    participant PView as TranslationPreviewView

    User->>View: フォルダ/ZIP選択
    View->>VM: SelectFolderCommand
    VM->>ML: LoadFromFolder(path)
    ML->>Parser: Parse(cfgStream)
    Parser-->>ML: CfgFile
    ML-->>VM: ModInfo
    VM-->>View: Mod情報を表示

    User->>View: 翻訳モード・言語を選択
    User->>View: 翻訳実行ボタン押下
    View->>VM: StartTranslationCommand

    VM->>Orch: ExecuteTranslation(mod, mode, src, tgt)

    loop 各 CfgEntry に対して
        Orch->>Vanilla: MatchByKey(section, key)
        alt キーマッチあり
            Vanilla-->>Orch: バニラ訳語
            Note over Orch: Source = VanillaKeyMatch
        else キーマッチなし
            Orch->>Vanilla: MatchByText(sourceText)
            alt テキストマッチあり
                Vanilla-->>Orch: バニラ訳語
                Note over Orch: Source = VanillaTextMatch
            else マッチなし
                Orch->>Glossary: ApplyGlossary(text)
                Glossary-->>Orch: 用語適用済みテキスト
                Orch->>Vanilla: GetContextHints(sourceText)
                Vanilla-->>Orch: 参考バニラ訳語リスト
                Orch->>Engine: TranslateAsync(text + hints)
                Engine-->>Orch: 翻訳結果
                Note over Orch: Source = API
            end
        end
        Orch->>History: SaveRecord(record)
    end

    Orch-->>VM: List~TranslationItem~
    VM-->>PView: プレビュー画面に遷移・表示
```

### 4.2 翻訳プレビュー → 保存フロー

```mermaid
sequenceDiagram
    actor User as ユーザー
    participant View as TranslationPreviewView
    participant VM as TranslationPreviewVM
    participant Parser as CfgParser
    participant FS as ファイルシステム

    View->>VM: Items表示
    User->>View: セルを編集
    View->>VM: TranslationItem.TranslatedText更新
    VM->>VM: IsEdited = true, Source = Manual

    User->>View: 保存ボタン押下
    View->>VM: SaveCommand

    VM->>VM: TranslationItemsをCfgFileに変換
    VM->>Parser: Write(cfgFile, stream)
    Parser->>FS: locale/{lang}/{mod}.cfg 書き出し
    FS-->>Parser: 完了
    Parser-->>VM: 完了
    VM-->>View: 保存完了通知
```

### 4.3 設定画面 - APIキー保存フロー

```mermaid
sequenceDiagram
    actor User as ユーザー
    participant View as SettingsView
    participant VM as SettingsVM
    participant Settings as SettingsService
    participant Engine as ITranslationEngine

    User->>View: APIキーを入力
    User->>View: テスト接続ボタン押下
    View->>VM: TestApiKeyCommand

    VM->>Engine: TranslateAsync("test", "en", "ja")
    alt 成功
        Engine-->>VM: 翻訳結果
        VM-->>View: "接続成功" 表示
    else 失敗
        Engine-->>VM: エラー
        VM-->>View: "接続失敗: {エラー}" 表示
    end

    User->>View: 保存ボタン押下
    View->>VM: SaveCommand
    VM->>Settings: SaveApiKey(engine, key)
    Note over Settings: DPAPIで暗号化して保存
    Settings-->>VM: 完了
    VM-->>View: 保存完了通知
```

---

## 5. 状態遷移図

### 5.1 アプリケーション全体の状態遷移

```mermaid
stateDiagram-v2
    [*] --> 初期化中
    初期化中 --> Mod未選択: 設定読込完了

    state "メイン画面" as Main {
        Mod未選択 --> Mod読込中: フォルダ/ZIP選択
        Mod読込中 --> Mod読込済: 読込成功
        Mod読込中 --> Mod未選択: 読込失敗

        Mod読込済 --> 翻訳実行中: 翻訳開始
        翻訳実行中 --> プレビュー表示: 翻訳完了
        翻訳実行中 --> Mod読込済: 翻訳失敗/キャンセル

        プレビュー表示 --> 編集中: セル編集
        編集中 --> プレビュー表示: 編集確定
        プレビュー表示 --> 保存中: 保存実行
        保存中 --> 保存完了: 保存成功
        保存完了 --> Mod未選択: 別のModを選択
        保存完了 --> プレビュー表示: 再編集
    }

    state "設定画面" as Settings {
        設定表示 --> 設定編集中: 値変更
        設定編集中 --> 設定表示: 保存/キャンセル
    }

    state "用語集画面" as Glossary {
        用語一覧 --> 用語編集中: 追加/編集
        用語編集中 --> 用語一覧: 保存/キャンセル
    }

    Main --> Settings: 設定タブ選択
    Settings --> Main: メインタブ選択
    Main --> Glossary: 用語集タブ選択
    Glossary --> Main: メインタブ選択
```

### 5.2 翻訳エントリの状態遷移

```mermaid
stateDiagram-v2
    [*] --> 未翻訳

    未翻訳 --> バニラ訳語適用: キー/テキストマッチ
    未翻訳 --> 用語集適用: 用語集マッチ
    未翻訳 --> API翻訳済: API翻訳実行
    未翻訳 --> 履歴復元: 履歴からの復元

    バニラ訳語適用 --> 手動編集済: ユーザー編集
    用語集適用 --> 手動編集済: ユーザー編集
    API翻訳済 --> 手動編集済: ユーザー編集
    履歴復元 --> 手動編集済: ユーザー編集

    手動編集済 --> API翻訳済: 再翻訳
    手動編集済 --> バニラ訳語適用: バニラ訳語にリバート

    バニラ訳語適用 --> 保存済: 保存
    用語集適用 --> 保存済: 保存
    API翻訳済 --> 保存済: 保存
    手動編集済 --> 保存済: 保存
    履歴復元 --> 保存済: 保存

    保存済 --> [*]
```

---

## 6. データフロー図

```mermaid
flowchart TB
    subgraph Input["入力"]
        ModFolder["Modフォルダ"]
        ModZip["Mod ZIPファイル"]
        FactorioDir["Factorioインストールフォルダ"]
        UserEdit["ユーザー手動入力"]
    end

    subgraph Processing["処理"]
        ModLoader["ModLoader\n(Mod読込)"]
        CfgParser["CfgParser\n(cfg解析/生成)"]
        VanillaService["VanillaService\n(バニラ訳語マッチング)"]
        Orchestrator["TranslationOrchestrator\n(翻訳実行制御)"]
        GlossaryService["GlossaryService\n(用語集適用)"]
    end

    subgraph External["外部API"]
        DeepL["DeepL API"]
        GoogleAPI["Google Translate API"]
    end

    subgraph Storage["永続化"]
        SQLite["SQLite\n(翻訳履歴)"]
        JSON_Settings["JSON\n(設定)"]
        JSON_Glossary["JSON\n(用語集)"]
        DPAPI["DPAPI暗号化\n(APIキー)"]
    end

    subgraph Output["出力"]
        CfgOutput["翻訳済 .cfg ファイル\n(locale/{lang}/*.cfg)"]
    end

    ModFolder --> ModLoader
    ModZip --> ModLoader
    ModLoader --> CfgParser
    FactorioDir --> VanillaService

    CfgParser --> Orchestrator
    VanillaService --> Orchestrator
    GlossaryService --> Orchestrator
    JSON_Glossary --> GlossaryService
    UserEdit --> Orchestrator

    Orchestrator --> DeepL
    Orchestrator --> GoogleAPI
    DeepL --> Orchestrator
    GoogleAPI --> Orchestrator

    Orchestrator --> SQLite
    Orchestrator --> CfgParser
    CfgParser --> CfgOutput

    JSON_Settings --> Orchestrator
    DPAPI --> Orchestrator
```

---

## 7. 画面構成

### 7.1 画面一覧

| 画面 | 概要 | 主要操作 |
|---|---|---|
| メインウィンドウ | TabControlベースの全体レイアウト | タブ切替、言語切替 |
| Mod選択タブ | Modの読込と翻訳実行 | フォルダ/ZIP選択、翻訳モード選択、翻訳実行 |
| プレビュータブ | 翻訳結果の確認・編集 | セル編集、フィルタ、保存 |
| 設定タブ | APIキーやパス等の設定 | APIキー入力、テスト接続、Factorioパス |
| 用語集タブ | 用語の登録・管理 | 追加、編集、削除、インポート/エクスポート |
| 履歴タブ | 翻訳履歴の参照 | 検索、フィルタ、再利用 |

### 7.2 画面遷移図

```mermaid
flowchart LR
    subgraph MainWindow
        Tab1["🔧 Mod選択"]
        Tab2["📝 プレビュー"]
        Tab3["⚙️ 設定"]
        Tab4["📖 用語集"]
        Tab5["📋 履歴"]
    end

    Tab1 -->|翻訳完了| Tab2
    Tab1 <-->|タブ切替| Tab3
    Tab1 <-->|タブ切替| Tab4
    Tab1 <-->|タブ切替| Tab5
    Tab2 <-->|タブ切替| Tab1
    Tab3 <-->|タブ切替| Tab1
```

---

## 8. データ定義

### 8.1 .cfg ファイルフォーマット

```ini
; コメント行
[section-name]
key1=Value text
key2=Another value with __1__ placeholders
key3=Multi-word value

[another-section]
key-a=Some text
```

**解析ルール:**
- `;` で始まる行はコメント
- `[xxx]` はセクションヘッダー
- `key=value` は翻訳エントリ（`=` の左がキー、右が値）
- `__1__`, `__2__` はプレースホルダー（翻訳時に保持必須）
- 空行はそのまま保持

### 8.2 SQLite テーブル定義

```sql
CREATE TABLE translation_history (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    mod_name        TEXT NOT NULL,
    mod_version     TEXT,
    section         TEXT NOT NULL,
    key             TEXT NOT NULL,
    source_lang     TEXT NOT NULL,
    target_lang     TEXT NOT NULL,
    source_text     TEXT NOT NULL,
    translated_text TEXT NOT NULL,
    engine          TEXT NOT NULL,
    translated_at   TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(mod_name, section, key, target_lang)
);

CREATE INDEX idx_history_mod ON translation_history(mod_name);
CREATE INDEX idx_history_key ON translation_history(section, key);
```

### 8.3 設定ファイル (appsettings.json)

```json
{
  "selectedEngine": "DeepL",
  "factorioInstallPath": "C:\\Program Files\\Factorio",
  "uiLanguage": "ja",
  "lastModPath": "",
  "windowWidth": 1200,
  "windowHeight": 800
}
```

### 8.4 用語集ファイル (glossary.json)

```json
[
  {
    "sourceTerm": "iron plate",
    "targetTerm": "鉄板",
    "sourceLang": "en",
    "targetLang": "ja",
    "excludeFromTranslation": false
  }
]
```

---

## 9. バニラ訳語マッチング アルゴリズム

```mermaid
flowchart TB
    Start["翻訳エントリ\n(section, key, sourceText)"]
    KeyMatch{"キーベースマッチング\nバニラに同一の\nsection+key が存在？"}
    TextMatch{"テキストベースマッチング\nバニラに同一の\nsourceText が存在？"}
    UseVanillaKey["バニラ訳語を適用\n(Source: VanillaKeyMatch)"]
    UseVanillaText["バニラ訳語を適用\n(Source: VanillaTextMatch)"]
    GetHints["バニラ訳語の参考情報を取得"]
    APITranslate["翻訳APIに送信\n(参考訳語付きプロンプト)"]
    UseAPI["API翻訳結果を適用\n(Source: API)"]
    End["TranslationItem生成"]

    Start --> KeyMatch
    KeyMatch -->|Yes| UseVanillaKey --> End
    KeyMatch -->|No| TextMatch
    TextMatch -->|Yes| UseVanillaText --> End
    TextMatch -->|No| GetHints --> APITranslate --> UseAPI --> End
```

### 優先度テーブル

| 優先度 | ソース | 条件 | 上書き可否 |
|---|---|---|---|
| 1 | 用語集 (Glossary) | 完全一致する用語が登録済 | ユーザー編集可 |
| 2 | バニラ (キーマッチ) | section+keyがバニラと一致 | ユーザー編集可 |
| 3 | バニラ (テキストマッチ) | 英語原文がバニラと一致 | ユーザー編集可 |
| 4 | 翻訳履歴 | 過去に同キーの翻訳あり | ユーザー編集可 |
| 5 | 翻訳API | API呼び出し（バニラ参考付き） | ユーザー編集可 |

---

## 10. 翻訳モード別処理

| モード | 対象エントリ | 既存翻訳の扱い | 用途 |
|---|---|---|---|
| 新規翻訳 | ソース言語の全エントリ | 翻訳先localeが無い前提 | Modの初回翻訳 |
| 差分翻訳 | ターゲットに存在しないエントリのみ | 既存翻訳は保持 | Modアップデート後の追加分翻訳 |
| 上書き更新 | ソース言語の全エントリ | 既存翻訳を上書き | 全体の再翻訳 |
| 手動編集 | なし（API呼び出しなし） | プレビュー画面で直接編集 | 微調整 |

---

## 11. エラーハンドリング方針

| カテゴリ | エラー例 | 対処 |
|---|---|---|
| API | APIキー無効、レート制限 | ユーザーに通知、リトライ提案 |
| ファイル | cfg解析失敗、書込権限なし | 詳細エラーメッセージ表示 |
| ネットワーク | 接続タイムアウト | リトライ（最大3回）、フォールバック |
| データ | SQLite破損 | DB再作成を提案 |

---

## 12. 非機能要件

| 項目 | 要件 |
|---|---|
| パフォーマンス | 1000エントリの翻訳を5分以内（API応答時間依存） |
| 応答性 | 翻訳中もUIがフリーズしない（async/await） |
| セキュリティ | APIキーはDPAPIで暗号化保存 |
| 保守性 | MVVM+DI構成、インターフェース分離 |
| 拡張性 | 翻訳エンジンの追加が容易（ITranslationEngine実装追加のみ） |
| i18n | UI文字列はリソースファイルで管理 |
