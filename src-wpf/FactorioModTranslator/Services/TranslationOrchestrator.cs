using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using FactorioModTranslator.Models;

namespace FactorioModTranslator.Services
{
    public class TranslationItem
    {
        public string Section { get; set; } = string.Empty;
        public string Key { get; set; } = string.Empty;
        public string SourceText { get; set; } = string.Empty;
        public string TranslatedText { get; set; } = string.Empty;
        public string? VanillaTranslation { get; set; }
        public TranslationSource Source { get; set; }
        public bool IsEdited { get; set; }
    }

    public class TranslationOrchestrator
    {
        private readonly ITranslationEngine _engine;
        private readonly VanillaTranslationService _vanilla;
        private readonly GlossaryService _glossary;
        private readonly TranslationHistoryService _history;

        public TranslationOrchestrator(
            ITranslationEngine engine,
            VanillaTranslationService vanilla,
            GlossaryService glossary,
            TranslationHistoryService history)
        {
            _engine = engine;
            _vanilla = vanilla;
            _glossary = glossary;
            _history = history;
        }

        public async Task<List<TranslationItem>> ExecuteTranslationAsync(
            ModInfo mod, 
            TranslationMode mode, 
            string sourceLang, 
            string targetLang,
            IProgress<double>? progress = null)
        {
            Log.Info($"ExecuteTranslationAsync started: mod={mod.Name}, mode={mode}, sourceLang={sourceLang}, targetLang={targetLang}");
            var results = new List<TranslationItem>();
            var sourceFiles = mod.LocaleFiles.Where(f => f.LanguageCode == sourceLang).ToList();
            var targetFiles = mod.LocaleFiles.Where(f => f.LanguageCode == targetLang).ToDictionary(f => Path.GetFileName(f.FilePath));

            int totalEntries = sourceFiles.Sum(f => f.Entries.Count);
            int processedCount = 0;
            Log.Info($"Total entries to process: {totalEntries} across {sourceFiles.Count} files.");

            foreach (var cfgFile in sourceFiles)
            {
                Log.Debug($"Processing file: {cfgFile.FilePath}");
                var targetFile = targetFiles.GetValueOrDefault(Path.GetFileName(cfgFile.FilePath));

                foreach (var entry in cfgFile.Entries)
                {
                    var item = new TranslationItem
                    {
                        Section = entry.Section,
                        Key = entry.Key,
                        SourceText = entry.Value
                    };

                    // 1. Check if we should skip (Diff mode)
                    if (mode == TranslationMode.DiffTranslation && targetFile != null)
                    {
                        var existing = targetFile.Entries.FirstOrDefault(e => e.Section == entry.Section && e.Key == entry.Key);
                        if (existing != null && !string.IsNullOrEmpty(existing.Value))
                        {
                            item.TranslatedText = existing.Value;
                            item.Source = TranslationSource.History; // Or "Existing"
                            results.Add(item);
                            processedCount++;
                            progress?.Report((double)processedCount / totalEntries);
                            continue;
                        }
                    }

                    // 2. Vanilla Match
                    string? vanillaMatch = _vanilla.MatchByKey(entry.Section, entry.Key);
                    if (vanillaMatch != null)
                    {
                        item.TranslatedText = vanillaMatch;
                        item.VanillaTranslation = vanillaMatch;
                        item.Source = TranslationSource.VanillaKeyMatch;
                    }
                    else
                    {
                        vanillaMatch = _vanilla.MatchByText(entry.Value);
                        if (vanillaMatch != null)
                        {
                            item.TranslatedText = vanillaMatch;
                            item.VanillaTranslation = vanillaMatch;
                            item.Source = TranslationSource.VanillaTextMatch;
                        }
                    }

                    // 3. History Match (if no vanilla match)
                    if (string.IsNullOrEmpty(item.TranslatedText))
                    {
                        var historyMatch = _history.GetPreviousTranslation(mod.Name, entry.Section, entry.Key, targetLang);
                        if (historyMatch != null)
                        {
                            item.TranslatedText = historyMatch;
                            item.Source = TranslationSource.History;
                        }
                    }

                    // 4. API Translation (if still no translation or Overwrite mode)
                    if (string.IsNullOrEmpty(item.TranslatedText) || mode == TranslationMode.OverwriteUpdate)
                    {
                        // Apply glossary first
                        string textToTranslate = _glossary.ApplyGlossary(entry.Value, sourceLang, targetLang);
                        
                        // Execute API call
                        item.TranslatedText = await _engine.TranslateAsync(textToTranslate, sourceLang, targetLang);
                        item.Source = TranslationSource.API;
                    }

                    results.Add(item);
                    
                    // Save to history
                    _history.SaveRecord(new TranslationRecord
                    {
                        ModName = mod.Name,
                        Section = entry.Section,
                        Key = entry.Key,
                        SourceLang = sourceLang,
                        TargetLang = targetLang,
                        SourceText = entry.Value,
                        TranslatedText = item.TranslatedText,
                        Engine = _engine.Name
                    });

                    processedCount++;
                    progress?.Report((double)processedCount / totalEntries);
                }
            }

            return results;
        }
    }
}
