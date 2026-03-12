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
            if (File.Exists(_filePath))
            {
                try
                {
                    string json = File.ReadAllText(_filePath);
                    _entries = JsonSerializer.Deserialize<List<GlossaryEntry>>(json) ?? new();
                }
                catch { _entries = new(); }
            }
        }

        public void Save()
        {
            string directory = Path.GetDirectoryName(_filePath)!;
            if (!Directory.Exists(directory)) Directory.CreateDirectory(directory);

            string json = JsonSerializer.Serialize(_entries, new JsonSerializerOptions { WriteIndented = true });
            File.WriteAllText(_filePath, json);
        }

        public void AddEntry(GlossaryEntry entry)
        {
            _entries.RemoveAll(e => e.SourceTerm == entry.SourceTerm && e.SourceLang == entry.SourceLang && e.TargetLang == entry.TargetLang);
            _entries.Add(entry);
            Save();
        }

        public void RemoveEntry(string sourceTerm)
        {
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
