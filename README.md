# Factorio Mod Translator

A GUI tool for Windows to automatically translate and manage Factorio 2.x (Space Age) Mod translation files.

If you find this software useful, please buy me a cup of coffee!

[![Buy Me a Coffee](https://img.buymeacoffee.com/button-api/?text=Buy%20me%20a%20coffee&emoji=&slug=taisuk46&button_colour=FFDD00&font_colour=000000&font_family=Cookie&outline_colour=000000&coffee_colour=ffffff)](https://www.buymeacoffee.com/taisuk46)

## Main Features

- **Mod Loading**: Directly load Mod folders.
- **Translation Engines**: Supports DeepL API.
- **Delta Translation**: Automatically extracts and translates only untranslated parts, preserving existing translations.
- **Translation History**: Saves past translations to an SQLite database for reuse.
- **Dark Theme**: A polished UI based on Factorio's orange and gray color scheme.

## Tech Stack

- Rust / Tauri 2.x
- Vanilla JavaScript / HTML / CSS (Frontend)
- reqwest (HTTP communication - DeepL API)
- rusqlite (SQLite - Translation History)
- keyring (API key storage - OS Credential Manager)
- tauri-plugin-log (Logging)

## Usage

1. Launch the app and set your translation API keys in the [Settings] tab.
2. In the [Mod Selection] tab, select the folder of the Mod you want to translate.
3. Click the translate button to execute.
4. Review/edit the results in the [Preview] tab and save.

## Build Instructions

### Prerequisites
- [Node.js](https://nodejs.org/) (LTS)
- [Rust](https://www.rust-lang.org/tools/install)
- [Tauri 2 CLI prerequisites](https://v2.tauri.app/start/prerequisites/)

### Setup
```bash
npm install
```

### Development Build (Debug)
Starts the application in development mode with hot-reloading enabled.

```bash
npm run tauri dev
```

### Release Build
Generates optimized executables and an NSIS installer.

```bash
npm run tauri build
```

### Build Artifacts
- **Executable**: `src-tauri/target/release/factoriomodtranslator.exe`
- **NSIS Installer**: `src-tauri/target/release/bundle/nsis/FactorioModTranslator_x.x.x_x64-setup.exe` (~3.5MB)

WebView2 runtime is pre-installed on Windows 10 and later, so no additional runtime installation is required.

## Logs

Application logs are output to:

- **Path**: `%LOCALAPPDATA%\FactorioModTranslator\logs\log_yyyyMMdd.log`
- **Format**: `[Timestamp] [Level] [File:Line] Message`

Both Rust backend and frontend logs are unified into this single file.

## License

MIT License
