# Factorio Mod Translator Onboarding

## Project Purpose
A Windows-based GUI tool for translating Factorio 2.x (Space Age) mods. It supports DeepL and Google Translate APIs and ensures consistency by prioritizing official Factorio translations (vanilla).

## Tech Stack
- **Language**: C# (.NET 8)
- **UI Framework**: WPF (Windows Presentation Foundation)
- **MVVM Framework**: CommunityToolkit.Mvvm
- **Database**: SQLite (Microsoft.Data.Sqlite) for history and glossary
- **External APIs**: DeepL.net, Google.Cloud.Translation.V2
- **Encryption**: System.Security.Cryptography.ProtectedData for API keys

## Codebase Structure
- `src/FactorioModTranslator`: Main application project.
    - `Models`: Data structures.
    - `ViewModels`: Business logic and UI state (MVVM).
    - `Views`: XAML UI definitions.
    - `Services`: Core logic (translation, persistence, Mod loading).
    - `Converters`: UI value converters.
- `tests/FactorioModTranslator.Tests`: Unit test project.

## Code Style & Conventions
- **Language**: C# 12
- **Naming**:
    - Classes, Methods, Public Properties: `PascalCase`
    - Private Fields: `_camelCase` (e.g., `_loc`)
    - Local Variables: `camelCase`
- **Patterns**: MVVM (using CommunityToolkit.Mvvm decorators like `[ObservableProperty]`).

## Development Commands
- **Build**: `dotnet build`
- **Test**: `dotnet test`
- **Clean**: `dotnet clean`

## Publishing
Refer to [README.md](file:///C:/Users/taisu/.gemini/antigravity/project/factoriomodtranslator_app/README.md) for detailed publish commands (SingleFile, SelfContained).
