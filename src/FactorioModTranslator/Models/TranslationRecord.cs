using System;

namespace FactorioModTranslator.Models
{
    /// <summary>
    /// Represents a record of a translation performed.
    /// </summary>
    public class TranslationRecord
    {
        public int Id { get; set; }
        public string ModName { get; set; } = string.Empty;
        public string Section { get; set; } = string.Empty;
        public string Key { get; set; } = string.Empty;
        public string SourceLang { get; set; } = string.Empty;
        public string TargetLang { get; set; } = string.Empty;
        public string SourceText { get; set; } = string.Empty;
        public string TranslatedText { get; set; } = string.Empty;
        public string Engine { get; set; } = string.Empty;
        public DateTime TranslatedAt { get; set; } = DateTime.Now;
    }
}
