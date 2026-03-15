# Factorio Mod Translator Onboarding

## Project Purpose
A Windows-based GUI tool for translating Factorio 2.x (Space Age) mods. It supports DeepL and Google Translate APIs and ensures consistency by prioritizing official Factorio translations (vanilla).

## Tech Stack
- **Frontend**: Vanilla JavaScript / HTML / CSS
- **Backend**: Rust
- **Framework**: Tauri 2.x (Rust + WebView2)
- **Database**: SQLite (rusqlite) for translation history
- **External APIs**: reqwest (HTTP) → DeepL API, Google Translate API
- **API Key Storage**: OS Keyring (`keyring` crate → Windows Credential Manager)
- **Logging**: tauri-plugin-log (`%LOCALAPPDATA%\FactorioModTranslator\logs`)
- **Distribution**: NSIS installer (.exe)

## Codebase Structure
- `src/`: Frontend (HTML/CSS/JS)
    - `index.html`, `index.css`, `styles.css`: UI
    - `main.js`: App logic (invoke() IPC calls to Rust)
    - `locales.json`: UI string resources (ja/en)
- `src-tauri/src/`: Rust backend
    - `commands/mod.rs`: IPC command definitions (AppState)
    - `models/`: Data structures (CfgEntry, CfgFile, ModInfo, etc.)
    - `services/`: Core logic (translation engines, cfg parser, mod loader, etc.)
    - `main.rs` / `lib.rs`: Entry point & Tauri app initialization

## Code Style & Conventions
- **Rust**: snake_case for functions/variables, PascalCase for types/structs
- **Architecture**: Command/IPC pattern (Frontend JS → invoke() → Rust commands)
- **Error Handling**: Result<T, String> returned from commands

## Development Commands
- **Dev Build**: `npm run tauri dev`
- **Release Build**: `npm run tauri build`
- **Rust Build**: `cd src-tauri && cargo build`
- **Rust Test**: `cd src-tauri && cargo test`

## Design Document
- [design_document.md](file:///c:/Users/taisu/.gemini/antigravity/project/factoriomodtranslator_app/docs/design_document.md)