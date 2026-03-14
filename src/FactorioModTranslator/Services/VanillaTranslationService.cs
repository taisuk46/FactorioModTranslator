using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using FactorioModTranslator.Models;

namespace FactorioModTranslator.Services
{
    public class VanillaTranslationService
    {
        private readonly CfgParser _cfgParser;
        private readonly Dictionary<string, string> _keyToValue = new();
        private readonly Dictionary<string, string> _textToValue = new();

        public VanillaTranslationService(CfgParser cfgParser)
        {
            _cfgParser = cfgParser;
        }

        /// <summary>
        /// Loads vanilla Factorio locale data from the installation path.
        /// </summary>
        /// <param name="factorioPath">Path to Factorio installation (e.g., C:\Program Files\Factorio)</param>
        /// <param name="langCode">Target language code (e.g., "ja")</param>
        public void LoadVanillaData(string factorioPath, string langCode)
        {
            _keyToValue.Clear();
            _textToValue.Clear();

            // Factorio 2.x data is split into base and space-age (and others)
            // Path: {factorioPath}/data/{module}/locale/{lang}/*.cfg
            string dataPath = Path.Combine(factorioPath, "data");
            if (!Directory.Exists(dataPath)) return;

            foreach (var module in Directory.GetDirectories(dataPath))
            {
                string localePath = Path.Combine(module, "locale", langCode);
                if (Directory.Exists(localePath))
                {
                    foreach (var cfgFile in Directory.GetFiles(localePath, "*.cfg"))
                    {
                        try
                        {
                            using var stream = File.OpenRead(cfgFile);
                            var parsed = _cfgParser.Parse(stream);
                            foreach (var entry in parsed.Entries)
                            {
                                string fullKey = $"{entry.Section}.{entry.Key}";
                                _keyToValue[fullKey] = entry.Value;

                                // We assumes English is the source. If we could load English too, we'd map EN text -> Lang text.
                                // For text-based matching, we need both English and the Target language.
                            }
                        }
                        catch { /* Ignore corrupted files */ }
                    }
                }
            }
            
            // To support text-based matching properly, we should also load English data
            LoadEnglishData(factorioPath);
        }

        private void LoadEnglishData(string factorioPath)
        {
            string dataPath = Path.Combine(factorioPath, "data");
            if (!Directory.Exists(dataPath)) return;

            foreach (var module in Directory.GetDirectories(dataPath))
            {
                string localePath = Path.Combine(module, "locale", "en");
                if (Directory.Exists(localePath))
                {
                    foreach (var cfgFile in Directory.GetFiles(localePath, "*.cfg"))
                    {
                        try
                        {
                            using var stream = File.OpenRead(cfgFile);
                            var parsed = _cfgParser.Parse(stream);
                            foreach (var entry in parsed.Entries)
                            {
                                string fullKey = $"{entry.Section}.{entry.Key}";
                                if (_keyToValue.TryGetValue(fullKey, out var targetValue))
                                {
                                    _textToValue[entry.Value] = targetValue;
                                }
                            }
                        }
                        catch { }
                    }
                }
            }
        }

        public virtual string? MatchByKey(string section, string key)
        {
            if (_keyToValue.TryGetValue($"{section}.{key}", out var value))
                return value;
            return null;
        }

        public virtual string? MatchByText(string sourceText)
        {
            if (_textToValue.TryGetValue(sourceText, out var value))
                return value;
            return null;
        }

        public virtual List<string> GetContextHints(string sourceText)
        {
            // Simple logic: return top 3 matches that contain the source text or are similar
            return _textToValue.Keys
                .Where(k => k.Contains(sourceText, StringComparison.OrdinalIgnoreCase))
                .Take(3)
                .Select(k => $"{k} → {_textToValue[k]}")
                .ToList();
        }
    }
}
