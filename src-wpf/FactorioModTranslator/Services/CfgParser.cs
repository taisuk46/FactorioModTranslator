using System;
using System.Collections.Generic;
using System.IO;
using System.Text;
using FactorioModTranslator.Models;

namespace FactorioModTranslator.Services
{
    /// <summary>
    /// Service for parsing and writing Factorio .cfg locale files.
    /// </summary>
    public class CfgParser
    {
        /// <summary>
        /// Parses a Factorio .cfg file from a stream.
        /// </summary>
        public CfgFile Parse(Stream stream, string filePath = "", string langCode = "")
        {
            Log.Debug($"CfgParser.Parse started: file={Path.GetFileName(filePath)}, lang={langCode}");
            var cfgFile = new CfgFile
            {
                FilePath = filePath,
                LanguageCode = langCode
            };

            using var reader = new StreamReader(stream, Encoding.UTF8);
            string? currentSection = null;
            var currentComments = new List<string>();
            bool isHeader = true;

            while (!reader.EndOfStream)
            {
                var line = reader.ReadLine()?.Trim();

                if (string.IsNullOrWhiteSpace(line))
                {
                    continue;
                }

                // Comment line
                if (line.StartsWith(";") || line.StartsWith("#"))
                {
                    currentComments.Add(line);
                    continue;
                }

                // Section header [section-name]
                if (line.StartsWith("[") && line.EndsWith("]"))
                {
                    currentSection = line.Substring(1, line.Length - 2);
                    if (!cfgFile.SectionOrder.Contains(currentSection))
                    {
                        cfgFile.SectionOrder.Add(currentSection);
                    }

                    if (isHeader && currentComments.Count > 0)
                    {
                        cfgFile.HeaderComments.AddRange(currentComments);
                        currentComments.Clear();
                    }
                    isHeader = false;
                    continue;
                }

                // Key=Value pair
                if (currentSection != null && line.Contains("="))
                {
                    isHeader = false;
                    var parts = line.Split('=', 2);
                    var entry = new CfgEntry
                    {
                        Section = currentSection,
                        Key = parts[0].Trim(),
                        Value = parts[1].Trim(),
                        Comment = currentComments.Count > 0 ? string.Join(Environment.NewLine, currentComments) : null
                    };
                    cfgFile.Entries.Add(entry);
                    currentComments.Clear();
                }
                else if (isHeader)
                {
                    // If we haven't hit a section yet, treat comments as header comments
                    cfgFile.HeaderComments.AddRange(currentComments);
                    currentComments.Clear();
                }
            }

            Log.Debug($"CfgParser.Parse completed: {cfgFile.Entries.Count} entries found.");
            return cfgFile;
        }

        /// <summary>
        /// Writes a CfgFile to a stream.
        /// </summary>
        public void Write(CfgFile cfgFile, Stream stream)
        {
            Log.Debug($"CfgParser.Write started: file={Path.GetFileName(cfgFile.FilePath)}, entries={cfgFile.Entries.Count}");
            using var writer = new StreamWriter(stream, new UTF8Encoding(false)); // Factorio expects UTF-8 without BOM usually

            // Write header comments
            foreach (var comment in cfgFile.HeaderComments)
            {
                writer.WriteLine(comment);
            }

            if (cfgFile.HeaderComments.Count > 0)
            {
                writer.WriteLine();
            }

            // Write by section
            foreach (var section in cfgFile.SectionOrder)
            {
                writer.WriteLine($"[{section}]");

                foreach (var entry in cfgFile.Entries)
                {
                    if (entry.Section == section)
                    {
                        if (!string.IsNullOrEmpty(entry.Comment))
                        {
                            writer.WriteLine(entry.Comment);
                        }
                        writer.WriteLine($"{entry.Key}={entry.Value}");
                    }
                }
                writer.WriteLine();
            }
            Log.Debug("CfgParser.Write completed.");
        }
    }
}
