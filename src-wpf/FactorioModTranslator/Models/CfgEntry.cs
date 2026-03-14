using System;

namespace FactorioModTranslator.Models
{
    /// <summary>
    /// Represents a single locale entry in a Factorio .cfg file.
    /// </summary>
    public class CfgEntry
    {
        /// <summary>
        /// The section name (e.g., "item-name") without brackets.
        /// </summary>
        public string Section { get; set; } = string.Empty;

        /// <summary>
        /// The key of the entry (e.g., "iron-plate").
        /// </summary>
        public string Key { get; set; } = string.Empty;

        /// <summary>
        /// The translated value of the entry.
        /// </summary>
        public string Value { get; set; } = string.Empty;

        /// <summary>
        /// Any comment line immediately preceding this entry, if any.
        /// </summary>
        public string? Comment { get; set; }
    }
}
