# Factorio Mod 自動翻訳ツール - タスク管理

## フェーズ1: 計画・環境構築
- [x] 技術調査（Factorio cfg形式、翻訳API SDK）
- [x] 実装計画書の作成とレビュー
- [x] .NET 8 SDKのインストール
- [x] WPFソリューション・プロジェクト作成
- [x] NuGetパッケージ追加

## フェーズ2: コア層（Models / Services）
- [x] cfg file parser (read/write)
- [x] Mod loading (folder/ZIP support)
- [x] 翻訳エンジンサービス（DeepL / Google）
- [x] バニラ翻訳データ取得・マッチングサービス
- [x] 用語集（グロッサリー）サービス
- [x] 設定管理サービス
- [x] 翻訳履歴サービス（SQLite）
- [x] APIキー管理（DPAPI暗号化）
- [x] 多言語UI（i18n）サービス
- [x] 翻訳オーケストレーター（全体統括）

## フェーズ3: ViewModel層
- [x] メインウィンドウViewModel
- [x] Mod選択 / 翻訳モード選択ViewModel
- [/] 翻訳プレビュー・編集ViewModel
- [x] 設定画面ViewModel
- [x] 用語集管理ViewModel

## フェーズ4: View層（WPF XAML）
- [x] メインウィンドウ
- [x] Mod選択画面
- [x] 翻訳プレビュー・編集画面
- [x] 設定画面
- [x] 用語集管理画面

## フェーズ5: 検証・仕上げ
- [x] ビルド・動作確認
- [x] ユニットテスト（cfgパーサー、翻訳サービス）
- [x] READMEとLICENSE作成

## フェーズ6: リリースビルド
- [x] 自己完結型シングルファイルexeのビルド
- [x] 配布用ZIPの作成

---
**開発完了!** 全5フェーズの実装とビルド確認が終了しました。
