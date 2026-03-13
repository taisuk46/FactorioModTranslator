# Suggested Commands

| Task | Command |
| --- | --- |
| Build Solution | `dotnet build` |
| Run Tests | `dotnet test` |
| Clean Project | `dotnet clean` |
| Debug Publish | `dotnet publish src/FactorioModTranslator/FactorioModTranslator.csproj -c Debug -r win-x64 --self-contained true -p:PublishSingleFile=true -p:PublishReadyToRun=false -p:IncludeNativeLibrariesForSelfExtract=true -o ./publish_debug` |
| Release Publish | `dotnet publish src/FactorioModTranslator/FactorioModTranslator.csproj -c Release -r win-x64 --self-contained true -p:PublishSingleFile=true -p:PublishReadyToRun=false -p:IncludeNativeLibrariesForSelfExtract=true -o ./publish` |
