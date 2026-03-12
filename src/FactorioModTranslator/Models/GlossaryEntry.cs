namespace FactorioModTranslator.Models
{
    /// <summary>
    /// Represents a glossary entry for consistent translation.
    /// </summary>
    public class GlossaryEntry
    {
        public string SourceTerm { get; set; } = string.Empty;
        public string TargetTerm { get; set; } = string.Empty;
        public string SourceLang { get; set; } = string.Empty;
        public string TargetLang { get; set; } = string.Empty;

        /// <summary>
        /// If true, this term should be excluded from translation (kept as original).
        /// </summary>
        public bool ExcludeFromTranslation { get; set; }
    }
}
