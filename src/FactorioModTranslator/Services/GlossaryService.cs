using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.Json;
using FactorioModTranslator.Models;

namespace FactorioModTranslator.Services
{
    public class GlossaryService
    {
        private List<GlossaryEntry> _entries = new();
        private readonly string _filePath;

        public GlossaryService(string? customPath = null)
        {
            string appData = Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData);
            _filePath = customPath ?? Path.Combine(appData, "FactorioModTranslator", "glossary.json");
            Load();
        }

        public void Load()
        {
            Log.Debug($"GlossaryService.Load started from: {_filePath}");
            if (File.Exists(_filePath))
            {
                try
                {
                    string json = File.ReadAllText(_filePath);
                    _entries = JsonSerializer.Deserialize<List<GlossaryEntry>>(json) ?? new();
                    Log.Info($"Loaded {_entries.Count} glossary entries.");
                }
                catch (Exception ex)
                {
                    Log.Error("Failed to load glossary. Initializing empty list.", ex);
                    _entries = new();
                }
            }
            else
            {
                Log.Info("Glossary file not found. Starting with empty glossary.");
                _entries = new();
            }
        }

        public void Save()
        {
            Log.Debug($"GlossaryService.Save started to: {_filePath}");
            try
            {
                string directory = Path.GetDirectoryName(_filePath)!;
                if (!Directory.Exists(directory)) Directory.CreateDirectory(directory);

                string json = JsonSerializer.Serialize(_entries, new JsonSerializerOptions { WriteIndented = true });
                File.WriteAllText(_filePath, json);
                Log.Info($"Saved {_entries.Count} glossary entries.");
            }
            catch (Exception ex)
            {
                Log.Error("Failed to save glossary", ex);
            }
        }

        public void AddEntry(GlossaryEntry entry)
        {
            Log.Info($"GlossaryService.AddEntry: {entry.SourceTerm} -> {entry.TargetTerm}");
            _entries.RemoveAll(e => e.SourceTerm == entry.SourceTerm && e.SourceLang == entry.SourceLang && e.TargetLang == entry.TargetLang);
            _entries.Add(entry);
            Save();
        }

        public void RemoveEntry(string sourceTerm)
        {
            Log.Info($"GlossaryService.RemoveEntry: {sourceTerm}");
            _entries.RemoveAll(e => e.SourceTerm == sourceTerm);
            Save();
        }

        public List<GlossaryEntry> GetAllEntries() => _entries.ToList();

        /// <summary>
        /// Applies glossary replacements to the text.
        /// </summary>
        public string ApplyGlossary(string text, string srcLang, string tgtLang)
        {
            foreach (var entry in _entries.Where(e => e.SourceLang == srcLang && e.TargetLang == tgtLang))
            {
                if (string.IsNullOrEmpty(entry.SourceTerm)) continue;
                
                // Case insensitive replacement
                text = System.Text.RegularExpressions.Regex.Replace(
                    text, 
                    System.Text.RegularExpressions.Regex.Escape(entry.SourceTerm), 
                    entry.TargetTerm, 
                    System.Text.RegularExpressions.RegexOptions.IgnoreCase);
            }
            return text;
        }
    }
}
