using System;
using System.Collections.Generic;

namespace FactorioModTranslator.Models
{
    /// <summary>
    /// Represents a Factorio .cfg locale file, preserving structure and comments.
    /// </summary>
    public class CfgFile
    {
        /// <summary>
        /// Original or intended file path.
        /// </summary>
        public string FilePath { get; set; } = string.Empty;

        /// <summary>
        /// Language code (e.g., "ja", "en").
        /// </summary>
        public string LanguageCode { get; set; } = string.Empty;

        /// <summary>
        /// All translation entries in the file.
        /// </summary>
        public List<CfgEntry> Entries { get; set; } = new();

        /// <summary>
        /// The order of sections as they appear in the file to preserve formatting.
        /// </summary>
        public List<string> SectionOrder { get; set; } = new();

        /// <summary>
        /// Header comments at the very top of the file.
        /// </summary>
        public List<string> HeaderComments { get; set; } = new();
    }
}
