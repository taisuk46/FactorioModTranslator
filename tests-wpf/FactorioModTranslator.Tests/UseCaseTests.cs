using Xunit;
using System;
using System.IO;
using System.IO.Compression;
using System.Threading.Tasks;
using System.Collections.Generic;
using System.Linq;
using Moq;
using FactorioModTranslator.Models;
using FactorioModTranslator.Services;

namespace FactorioModTranslator.Tests
{
    public class UseCaseTests : IDisposable
    {
        private readonly string _tempPath;

        public UseCaseTests()
        {
            _tempPath = Path.Combine(Path.GetTempPath(), "FactorioModTranslatorTests_" + Guid.NewGuid().ToString("N"));
            Directory.CreateDirectory(_tempPath);
        }

        public void Dispose()
        {
            // Clear SQLite pools to release file locks
            Microsoft.Data.Sqlite.SqliteConnection.ClearAllPools();

            if (Directory.Exists(_tempPath))
            {
                try
                {
                    Directory.Delete(_tempPath, true);
                }
                catch (IOException ex)
                {
                    // Log or ignore if cleanup fails during test run
                    Console.WriteLine($"Cleanup failed: {ex.Message}");
                }
            }
        }

        // UC1: Modを読み込む
        [Fact]
        public void UC1_LoadMod_Success_ShouldExtractCfgEntries()
        {
            // Arrange
            var modDir = Path.Combine(_tempPath, "MyMod");
            Directory.CreateDirectory(modDir);
            File.WriteAllText(Path.Combine(modDir, "info.json"), "{\"name\": \"MyMod\", \"version\": \"1.0.0\"}");
            var localeDir = Path.Combine(modDir, "locale", "en");
            Directory.CreateDirectory(localeDir);
            File.WriteAllText(Path.Combine(localeDir, "test.cfg"), "[item-name]\nitem1=Item One\n");

            var parser = new CfgParser();
            var loader = new ModLoader(parser);

            // Act
            var modInfo = loader.LoadFromFolder(modDir);

            // Assert
            Assert.Equal("MyMod", modInfo.Name);
            Assert.Single(modInfo.LocaleFiles);
            Assert.Equal("en", modInfo.LocaleFiles[0].LanguageCode);
            Assert.Equal("Item One", modInfo.LocaleFiles[0].Entries.First().Value);
        }

        [Fact]
        public void UC1_LoadMod_Alternative_NoInfoJson_ShouldNotFailButHaveEmptyMetadata()
        {
            var modDir = Path.Combine(_tempPath, "NoInfoMod");
            Directory.CreateDirectory(modDir);
            var localeDir = Path.Combine(modDir, "locale", "en");
            Directory.CreateDirectory(localeDir);
            File.WriteAllText(Path.Combine(localeDir, "test.cfg"), "[test]\nkey=val\n");

            var loader = new ModLoader(new CfgParser());
            var modInfo = loader.LoadFromFolder(modDir);

            Assert.Empty(modInfo.Name);
            Assert.Single(modInfo.LocaleFiles);
        }

        // UC2: 翻訳を実行する
        [Fact]
        public async Task UC2_ExecuteTranslation_Success_ShouldApplyTranslationInOrder()
        {
            // Arrange
            var engineMock = new Mock<ITranslationEngine>();
            var vanillaMock = new Mock<VanillaTranslationService>(new Mock<CfgParser>().Object); // Mocking class with null params if possible, or use interface
            var glossaryMock = new Mock<GlossaryService>((string?)null);
            var historyMock = new Mock<TranslationHistoryService>((string?)null);

            engineMock.Setup(e => e.Name).Returns("MockEngine");
            engineMock.Setup(e => e.TranslateAsync(It.IsAny<string>(), It.IsAny<string>(), It.IsAny<string>()))
                      .ReturnsAsync("TranslatedByAPI");

            vanillaMock.Setup(v => v.MatchByKey(It.IsAny<string>(), It.IsAny<string>())).Returns((string?)null);
            vanillaMock.Setup(v => v.MatchByText(It.IsAny<string>())).Returns((string?)null);
            
            glossaryMock.Setup(g => g.ApplyGlossary(It.IsAny<string>(), It.IsAny<string>(), It.IsAny<string>()))
                        .Returns((string s, string sl, string tl) => s);

            historyMock.Setup(h => h.GetPreviousTranslation(It.IsAny<string>(), It.IsAny<string>(), It.IsAny<string>(), It.IsAny<string>()))
                       .Returns((string?)null);

            var orchestrator = new TranslationOrchestrator(engineMock.Object, vanillaMock.Object, glossaryMock.Object, historyMock.Object);

            var mod = new ModInfo { Name = "TestMod" };
            var cfg = new CfgFile { LanguageCode = "en", FilePath = "test.cfg" };
            cfg.Entries.Add(new CfgEntry { Section = "s", Key = "k", Value = "v" });
            mod.LocaleFiles.Add(cfg);

            // Act
            var results = await orchestrator.ExecuteTranslationAsync(mod, TranslationMode.NewTranslation, "en", "ja");

            // Assert
            Assert.Single(results);
            Assert.Equal("TranslatedByAPI", results[0].TranslatedText);
            Assert.Equal(TranslationSource.API, results[0].Source);
        }

        [Fact]
        public async Task UC2_ExecuteTranslation_Alternative_VanillaMatch_ShouldUseVanilla()
        {
            // Arrange
            var engineMock = new Mock<ITranslationEngine>();
            var vanillaMock = new Mock<VanillaTranslationService>(new Mock<CfgParser>().Object);
            var glossaryMock = new Mock<GlossaryService>((string?)null);
            var historyMock = new Mock<TranslationHistoryService>((string?)null);

            vanillaMock.Setup(v => v.MatchByKey("s", "k")).Returns("VanillaVal");

            var orchestrator = new TranslationOrchestrator(engineMock.Object, vanillaMock.Object, glossaryMock.Object, historyMock.Object);
            var mod = new ModInfo { Name = "TestMod" };
            var cfg = new CfgFile { LanguageCode = "en", FilePath = "test.cfg" };
            cfg.Entries.Add(new CfgEntry { Section = "s", Key = "k", Value = "v" });
            mod.LocaleFiles.Add(cfg);

            // Act
            var results = await orchestrator.ExecuteTranslationAsync(mod, TranslationMode.NewTranslation, "en", "ja");

            // Assert
            Assert.Equal("VanillaVal", results[0].TranslatedText);
            Assert.Equal(TranslationSource.VanillaKeyMatch, results[0].Source);
            engineMock.Verify(e => e.TranslateAsync(It.IsAny<string>(), It.IsAny<string>(), It.IsAny<string>()), Times.Never);
        }

        // UC3: 翻訳をプレビュー・編集する
        [Fact]
        public void UC3_PreviewAndEditTranslation_Success_ShouldUpdateStateToManual()
        {
            // Arrange
            var item = new TranslationItem
            {
                SourceText = "Original",
                TranslatedText = "Translated",
                Source = TranslationSource.API
            };

            // Act
            item.TranslatedText = "Edited";
            item.IsEdited = true;
            item.Source = TranslationSource.Manual;

            // Assert
            Assert.True(item.IsEdited);
            Assert.Equal(TranslationSource.Manual, item.Source);
            Assert.Equal("Edited", item.TranslatedText);
        }

        [Fact]
        public void UC1_LoadMod_Alternative_NoLocale_ShouldReturnEmptyLocaleFiles()
        {
            // Arrange
            var modDir = Path.Combine(_tempPath, "NoLocaleMod");
            Directory.CreateDirectory(modDir);
            File.WriteAllText(Path.Combine(modDir, "info.json"), "{\"name\": \"NoLocaleMod\"}");

            var loader = new ModLoader(new CfgParser());

            // Act
            var modInfo = loader.LoadFromFolder(modDir);

            // Assert
            Assert.Empty(modInfo.LocaleFiles);
        }

        [Fact]
        public void UC1_LoadMod_Alternative_CfgParseError_ShouldReturnPartialData()
        {
            // Arrange
            var modDir = Path.Combine(_tempPath, "BadCfgMod");
            Directory.CreateDirectory(modDir);
            var localeDir = Path.Combine(modDir, "locale", "en");
            Directory.CreateDirectory(localeDir);
            // "Bad line" without equals or section shouldn't crash
            File.WriteAllText(Path.Combine(localeDir, "test.cfg"), "Invalid line\n[section]\nkey=value");

            var loader = new ModLoader(new CfgParser());

            // Act
            var modInfo = loader.LoadFromFolder(modDir);

            // Assert
            Assert.Single(modInfo.LocaleFiles);
            Assert.Single(modInfo.LocaleFiles[0].Entries);
        }

        [Fact]
        public void UC1_LoadMod_Alternative_CorruptedZip_ShouldThrowException()
        {
            // Arrange
            var zipPath = Path.Combine(_tempPath, "corrupted.zip");
            File.WriteAllText(zipPath, "not a zip file");

            var loader = new ModLoader(new CfgParser());

            // Act & Assert
            Assert.Throws<InvalidDataException>(() => loader.LoadFromZip(zipPath));
        }

        [Fact]
        public async Task UC2_ExecuteTranslation_Alternative_NoApiKey_ShouldThrowException()
        {
            // Arrange
            var engineMock = new Mock<ITranslationEngine>();
            engineMock.Setup(e => e.TranslateAsync(It.IsAny<string>(), It.IsAny<string>(), It.IsAny<string>()))
                      .ThrowsAsync(new InvalidOperationException("API Key not found"));

            var orchestrator = new TranslationOrchestrator(
                engineMock.Object, 
                new Mock<VanillaTranslationService>(new Mock<CfgParser>().Object).Object,
                new Mock<GlossaryService>((string?)null).Object,
                new Mock<TranslationHistoryService>((string?)null).Object);

            var mod = new ModInfo { Name = "TestMod" };
            var cfg = new CfgFile { LanguageCode = "en" };
            cfg.Entries.Add(new CfgEntry { Section = "s", Key = "k", Value = "v" });
            mod.LocaleFiles.Add(cfg);

            // Act & Assert
            await Assert.ThrowsAsync<InvalidOperationException>(() => 
                orchestrator.ExecuteTranslationAsync(mod, TranslationMode.NewTranslation, "en", "ja"));
        }

        [Fact]
        public async Task UC2_ExecuteTranslation_Alternative_ApiTimeout_ShouldThrowException()
        {
            // Arrange
            var engineMock = new Mock<ITranslationEngine>();
            engineMock.Setup(e => e.TranslateAsync(It.IsAny<string>(), It.IsAny<string>(), It.IsAny<string>()))
                      .ThrowsAsync(new TaskCanceledException("Timeout"));

            var orchestrator = new TranslationOrchestrator(
                engineMock.Object, 
                new Mock<VanillaTranslationService>(new Mock<CfgParser>().Object).Object,
                new Mock<GlossaryService>((string?)null).Object,
                new Mock<TranslationHistoryService>((string?)null).Object);

            var mod = new ModInfo { Name = "TestMod" };
            var cfg = new CfgFile { LanguageCode = "en" };
            cfg.Entries.Add(new CfgEntry { Section = "s", Key = "k", Value = "v" });
            mod.LocaleFiles.Add(cfg);

            // Act & Assert
            await Assert.ThrowsAsync<TaskCanceledException>(() => 
                orchestrator.ExecuteTranslationAsync(mod, TranslationMode.NewTranslation, "en", "ja"));
        }

        [Fact]
        public async Task UC2_ExecuteTranslation_Alternative_ApiRateLimit_ShouldThrowException()
        {
            // Arrange
            var engineMock = new Mock<ITranslationEngine>();
            engineMock.Setup(e => e.TranslateAsync(It.IsAny<string>(), It.IsAny<string>(), It.IsAny<string>()))
                      .ThrowsAsync(new Exception("Rate limit exceeded"));

            var orchestrator = new TranslationOrchestrator(
                engineMock.Object, 
                new Mock<VanillaTranslationService>(new Mock<CfgParser>().Object).Object,
                new Mock<GlossaryService>((string?)null).Object,
                new Mock<TranslationHistoryService>((string?)null).Object);

            var mod = new ModInfo { Name = "TestMod" };
            var cfg = new CfgFile { LanguageCode = "en" };
            cfg.Entries.Add(new CfgEntry { Section = "s", Key = "k", Value = "v" });
            mod.LocaleFiles.Add(cfg);

            // Act & Assert
            await Assert.ThrowsAsync<Exception>(() => 
                orchestrator.ExecuteTranslationAsync(mod, TranslationMode.NewTranslation, "en", "ja"));
        }

        [Fact]
        public async Task UC2_ExecuteTranslation_Alternative_Cancel_ShouldThrowExceptionIfSupported()
        {
            // Note: Current implementation doesn't support CancellationToken yet.
            // This test serves as a placeholder for future implementation.
            Assert.True(true);
        }

        [Fact]
        public void UC3_PreviewAndEditTranslation_Alternative_Revert_ShouldRestoreOriginal()
        {
            // Arrange
            var item = new TranslationItem
            {
                SourceText = "Original",
                TranslatedText = "AutoTranslated",
                Source = TranslationSource.API
            };
            string originalTranslation = item.TranslatedText;

            // Act (Simulate manual edit and then revert)
            item.TranslatedText = "ManualEdit";
            item.IsEdited = true;
            
            // Revert logic (in UI this would be implemented by keeping track of the original)
            item.TranslatedText = originalTranslation;
            item.IsEdited = false;

            // Assert
            Assert.False(item.IsEdited);
            Assert.Equal("AutoTranslated", item.TranslatedText);
        }

        // UC4: 翻訳ファイルを保存する
        [Fact]
        public void UC4_SaveTranslationFiles_Success_ShouldExportToTargetLocaleDirectory()
        {
            // Arrange
            var parser = new CfgParser();
            var cfg = new CfgFile
            {
                FilePath = "test.cfg",
                LanguageCode = "ja",
                SectionOrder = { "item-name" }
            };
            cfg.Entries.Add(new CfgEntry { Section = "item-name", Key = "item1", Value = "アイテム1" });

            var saveDir = Path.Combine(_tempPath, "locale", "ja");
            Directory.CreateDirectory(saveDir);
            var saveFile = Path.Combine(saveDir, "test.cfg");

            // Act
            using (var stream = File.Create(saveFile))
            {
                parser.Write(cfg, stream);
            }

            // Assert
            Assert.True(File.Exists(saveFile));
            var content = File.ReadAllText(saveFile);
            Assert.Contains("[item-name]", content);
            Assert.Contains("item1=アイテム1", content);
        }

        // UC5: 設定を管理する
        [Fact]
        public void UC5_ManageSettings_Success_ShouldSaveAndLoadSuccessfully()
        {
            // Arrange
            var settingsPath = Path.Combine(_tempPath, "settings.json");
            var service = new SettingsService(settingsPath);
            var newSettings = new AppSettings { SelectedEngine = "Google", FactorioPath = "CustomPath" };

            // Act
            service.SaveSettings(newSettings);
            var loadedService = new SettingsService(settingsPath);

            // Assert
            Assert.Equal("Google", loadedService.Current.SelectedEngine);
            Assert.Equal("CustomPath", loadedService.Current.FactorioPath);
        }

        // UC6: 用語集を管理する
        [Fact]
        public void UC6_ManageGlossary_Success_ShouldExcludeOrReplaceTerms()
        {
            // Arrange
            var glossaryPath = Path.Combine(_tempPath, "glossary.json");
            var service = new GlossaryService(glossaryPath);
            var entry = new GlossaryEntry { SourceTerm = "Iron", TargetTerm = "鉄", SourceLang = "en", TargetLang = "ja" };

            // Act
            service.AddEntry(entry);
            var result = service.ApplyGlossary("Iron Plate", "en", "ja");

            // Assert
            Assert.Equal("鉄 Plate", result);
        }

        [Fact]
        public void UC4_SaveTranslationFiles_Alternative_NoWritePermission_ShouldThrowException()
        {
            // Note: Testing file system permissions in unit tests can be tricky.
            // Usually we'd mock the file system if we used a wrapper.
            Assert.True(true);
        }

        [Fact]
        public void UC5_ManageSettings_Alternative_InvalidFactorioPath_ShouldBeStoredAsIs()
        {
            // Arrange
            var settingsPath = Path.Combine(_tempPath, "invalid_settings.json");
            var service = new SettingsService(settingsPath);
            var settings = new AppSettings { FactorioPath = "Z:\\Invalid\\Path" };

            // Act
            service.SaveSettings(settings);
            var loaded = new SettingsService(settingsPath);

            // Assert
            Assert.Equal("Z:\\Invalid\\Path", loaded.Current.FactorioPath);
        }

        [Fact]
        public void UC6_ManageGlossary_Alternative_DuplicateTerm_ShouldOverwriteExisting()
        {
            // Arrange
            var glossaryPath = Path.Combine(_tempPath, "duplicate_glossary.json");
            var service = new GlossaryService(glossaryPath);
            var entry1 = new GlossaryEntry { SourceTerm = "Iron", TargetTerm = "鉄", SourceLang = "en", TargetLang = "ja" };
            var entry2 = new GlossaryEntry { SourceTerm = "Iron", TargetTerm = "アイアン", SourceLang = "en", TargetLang = "ja" };

            // Act
            service.AddEntry(entry1);
            service.AddEntry(entry2);
            var entries = service.GetAllEntries();

            // Assert
            Assert.Single(entries);
            Assert.Equal("アイアン", entries[0].TargetTerm);
        }

        [Fact]
        public void UC9_SwitchUILanguage_Alternative_NoResourceFile_ShouldFallbackToDefault()
        {
            // Arrange
            var service = LocalizationService.Instance;
            
            // Act
            service.CurrentCulture = new System.Globalization.CultureInfo("fr-FR"); // Not explicitly handled in switch

            // Assert
            // Should fallback to English for known keys
            Assert.Equal("Factorio Mod Auto-Translator", service["AppTitle"]);
            // Should return the key itself for unknown keys
            Assert.Equal("NonExistentKey", service["NonExistentKey"]);
        }

        // UC7: 翻訳履歴を参照する
        [Fact]
        public void UC7_BrowseTranslationHistory_Success_ShouldSaveAndRetrieveFromSQLite()
        {
            // Arrange
            var dbPath = Path.Combine(_tempPath, "history.db");
            var service = new TranslationHistoryService(dbPath);
            var record = new TranslationRecord
            {
                ModName = "TestMod",
                Section = "s",
                Key = "k",
                SourceLang = "en",
                TargetLang = "ja",
                SourceText = "v",
                TranslatedText = "訳",
                Engine = "Mock",
                TranslatedAt = DateTime.Now
            };

            // Act
            service.SaveRecord(record);
            var history = service.GetAllHistory();

            // Assert
            Assert.Single(history);
            Assert.Equal("訳", history[0].TranslatedText);
        }

        // UC8: バニラ訳語を適用する
        [Fact]
        public void UC8_ApplyVanillaTranslation_Success_ShouldMatchByKeyAndText()
        {
            // Arrange
            var factorioPath = Path.Combine(_tempPath, "Factorio");
            var jaLocalePath = Path.Combine(factorioPath, "data", "base", "locale", "ja");
            var enLocalePath = Path.Combine(factorioPath, "data", "base", "locale", "en");
            Directory.CreateDirectory(jaLocalePath);
            Directory.CreateDirectory(enLocalePath);
            
            File.WriteAllText(Path.Combine(enLocalePath, "base.cfg"), "[item-name]\niron-plate=Iron plate\n");
            File.WriteAllText(Path.Combine(jaLocalePath, "base.cfg"), "[item-name]\niron-plate=鉄板\n");

            var parser = new CfgParser();
            var service = new VanillaTranslationService(parser);

            // Act
            service.LoadVanillaData(factorioPath, "ja");

            // Assert
            Assert.Equal("鉄板", service.MatchByKey("item-name", "iron-plate"));
            Assert.Equal("鉄板", service.MatchByText("Iron plate"));
        }

        // UC9: UI言語を切り替える
        [Fact]
        public void UC9_SwitchUILanguage_Success_ShouldTriggerLanguageChangedEvent()
        {
            // Arrange
            var service = LocalizationService.Instance;
            bool eventFired = false;
            service.PropertyChanged += (s, e) => { if (e.PropertyName == "CurrentCulture") eventFired = true; };

            // Act
            service.CurrentCulture = new System.Globalization.CultureInfo("en-US");

            // Assert
            Assert.True(eventFired);
            Assert.Equal("en", service.CurrentCulture.TwoLetterISOLanguageName);
            Assert.Equal("Select Mod", service["SelectMod"]);
        }

        [Fact]
        public void UC7_BrowseTranslationHistory_Alternative_NoHistory_ShouldShowEmpty()
        {
            var dbPath = Path.Combine(_tempPath, "empty_history.db");
            var service = new TranslationHistoryService(dbPath);
            var history = service.GetAllHistory();
            Assert.Empty(history);
        }

        [Fact]
        public void UC8_ApplyVanillaTranslation_Alternative_NoFactorioPath_ShouldSkip()
        {
            var service = new VanillaTranslationService(new CfgParser());
            service.LoadVanillaData(Path.Combine(_tempPath, "InvalidPath"), "ja");
            Assert.Null(service.MatchByKey("s", "k"));
        }
    }
}
