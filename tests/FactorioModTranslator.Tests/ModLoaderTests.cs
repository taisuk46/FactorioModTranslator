using System.IO;
using System.IO.Compression;
using System.Linq;
using FactorioModTranslator.Services;
using Xunit;

namespace FactorioModTranslator.Tests
{
    public class ModLoaderTests
    {
        [Fact]
        public void LoadFromFolder_ShouldLoadModInfo()
        {
            // Arrange
            string tempDir = Path.Combine(Path.GetTempPath(), "testmod_1.0.0");
            if (Directory.Exists(tempDir)) Directory.Delete(tempDir, true);
            Directory.CreateDirectory(tempDir);
            
            string infoJson = "{\"name\": \"testmod\", \"version\": \"1.0.0\", \"title\": \"Test Mod\"}";
            File.WriteAllText(Path.Combine(tempDir, "info.json"), infoJson);
            
            string localeDir = Path.Combine(tempDir, "locale", "en");
            Directory.CreateDirectory(localeDir);
            File.WriteAllText(Path.Combine(localeDir, "test.cfg"), "[item-name]\nitem1=Item 1");

            var parser = new CfgParser();
            var loader = new ModLoader(parser);

            // Act
            var modInfo = loader.LoadFromFolder(tempDir);

            // Assert
            Assert.Equal("testmod", modInfo.Name);
            Assert.Equal("1.0.0", modInfo.Version);
            Assert.Single(modInfo.LocaleFiles);
            Assert.Equal("en", modInfo.LocaleFiles[0].LanguageCode);

            // Cleanup
            Directory.Delete(tempDir, true);
        }

        [Fact]
        public void LoadFromZip_ShouldLoadModInfo()
        {
            // Arrange
            string tempDir = Path.Combine(Path.GetTempPath(), "zip_test_source");
            if (Directory.Exists(tempDir)) Directory.Delete(tempDir, true);
            Directory.CreateDirectory(Path.Combine(tempDir, "testmod_1.0.0", "locale", "en"));
            
            File.WriteAllText(Path.Combine(tempDir, "testmod_1.0.0", "info.json"), "{\"name\": \"testmod\"}");
            File.WriteAllText(Path.Combine(tempDir, "testmod_1.0.0", "locale", "en", "test.cfg"), "[item-name]\nitem1=Item 1");

            string zipPath = Path.Combine(Path.GetTempPath(), "testmod_1.0.0.zip");
            if (File.Exists(zipPath)) File.Delete(zipPath);
            ZipFile.CreateFromDirectory(tempDir, zipPath);

            var parser = new CfgParser();
            var loader = new ModLoader(parser);

            // Act
            var modInfo = loader.LoadFromZip(zipPath);

            // Assert
            Assert.Equal("testmod", modInfo.Name);
            Assert.Single(modInfo.LocaleFiles);

            // Cleanup
            Directory.Delete(tempDir, true);
            File.Delete(zipPath);
        }
    }
}
