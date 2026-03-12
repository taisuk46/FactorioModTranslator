using System;
using System.Collections.Generic;
using System.IO;
using System.IO.Compression;
using System.Linq;
using System.Text.Json;
using FactorioModTranslator.Models;

namespace FactorioModTranslator.Services
{
    public class ModLoader
    {
        private readonly CfgParser _cfgParser;

        public ModLoader(CfgParser cfgParser)
        {
            _cfgParser = cfgParser;
        }

        public ModInfo LoadFromFolder(string path)
        {
            if (!Directory.Exists(path))
                throw new DirectoryNotFoundException($"Directory not found: {path}");

            var modInfo = new ModInfo
            {
                SourcePath = path,
                SourceType = ModSourceType.Folder
            };

            // Read info.json
            string infoPath = Path.Combine(path, "info.json");
            if (File.Exists(infoPath))
            {
                PopulateModMetadata(modInfo, File.ReadAllText(infoPath));
            }

            // Find locale folder
            string localePath = Path.Combine(path, "locale");
            if (Directory.Exists(localePath))
            {
                foreach (var langDir in Directory.GetDirectories(localePath))
                {
                    string langCode = Path.GetFileName(langDir);
                    foreach (var cfgFile in Directory.GetFiles(langDir, "*.cfg"))
                    {
                        using var stream = File.OpenRead(cfgFile);
                        modInfo.LocaleFiles.Add(_cfgParser.Parse(stream, cfgFile, langCode));
                    }
                }
            }

            return modInfo;
        }

        public ModInfo LoadFromZip(string zipPath)
        {
            if (!File.Exists(zipPath))
                throw new FileNotFoundException($"Zip file not found: {zipPath}");

            var modInfo = new ModInfo
            {
                SourcePath = zipPath,
                SourceType = ModSourceType.Zip
            };

            using var archive = ZipFile.OpenRead(zipPath);
            
            // Factorio mods in zip usually have a top-level folder: modname_version/
            var firstEntry = archive.Entries.FirstOrDefault();
            if (firstEntry == null) throw new Exception("Zip is empty");

            string rootFolder = firstEntry.FullName.Split('/')[0] + "/";

            // metadata
            var infoEntry = archive.GetEntry(rootFolder + "info.json");
            if (infoEntry != null)
            {
                using var reader = new StreamReader(infoEntry.Open());
                PopulateModMetadata(modInfo, reader.ReadToEnd());
            }

            // locale
            var localeEntries = archive.Entries.Where(e => e.FullName.StartsWith(rootFolder + "locale/") && e.FullName.EndsWith(".cfg"));
            foreach (var entry in localeEntries)
            {
                // Parts: rootFolder/locale/langCode/filename.cfg
                var parts = entry.FullName.Split('/');
                if (parts.Length >= 4)
                {
                    string langCode = parts[parts.Length - 2];
                    using var stream = entry.Open();
                    modInfo.LocaleFiles.Add(_cfgParser.Parse(stream, entry.FullName, langCode));
                }
            }

            return modInfo;
        }

        private void PopulateModMetadata(ModInfo modInfo, string json)
        {
            try
            {
                var options = new JsonSerializerOptions { PropertyNameCaseInsensitive = true };
                var doc = JsonDocument.Parse(json);
                var root = doc.RootElement;

                if (root.TryGetProperty("name", out var name)) modInfo.Name = name.GetString() ?? "";
                if (root.TryGetProperty("version", out var version)) modInfo.Version = version.GetString() ?? "";
                if (root.TryGetProperty("title", out var title)) modInfo.Title = title.GetString() ?? "";
                if (root.TryGetProperty("author", out var author)) modInfo.Author = author.GetString() ?? "";
                if (root.TryGetProperty("factorio_version", out var fv)) modInfo.FactorioVersion = fv.GetString() ?? "";
            }
            catch (Exception ex)
            {
                // Log or handle JSON error
                Console.WriteLine($"Error parsing info.json: {ex.Message}");
            }
        }
    }
}
