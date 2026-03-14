using System.Collections.Generic;

namespace FactorioModTranslator.Models
{
    /// <summary>
    /// Represents information about a Factorio Mod.
    /// </summary>
    public class ModInfo
    {
        public string Name { get; set; } = string.Empty;
        public string Version { get; set; } = string.Empty;
        public string Title { get; set; } = string.Empty;
        public string Author { get; set; } = string.Empty;
        public string FactorioVersion { get; set; } = string.Empty;
        public string SourcePath { get; set; } = string.Empty;
        public ModSourceType SourceType { get; set; }

        /// <summary>
        /// Collection of locale configuration files found in the mod.
        /// </summary>
        public List<CfgFile> LocaleFiles { get; set; } = new();
    }
}
