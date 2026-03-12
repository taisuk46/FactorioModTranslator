using System.IO;
using System.Text;
using FactorioModTranslator.Models;
using FactorioModTranslator.Services;
using Xunit;

namespace FactorioModTranslator.Tests
{
    public class CfgParserTests
    {
        [Fact]
        public void Parse_ShouldCorrectlyParseCfgContent()
        {
            // Arrange
            var cfgContent = @"
; Header Comment
[item-name]
; Item Comment
iron-plate=鉄板
copper-plate=銅板

[entity-name]
small-biter=小型バイター
";
            var parser = new CfgParser();
            using var stream = new MemoryStream(Encoding.UTF8.GetBytes(cfgContent));

            // Act
            var result = parser.Parse(stream);

            // Assert
            Assert.Equal(3, result.Entries.Count);
            Assert.Contains("item-name", result.SectionOrder);
            Assert.Contains("entity-name", result.SectionOrder);
            
            var ironPlate = result.Entries.Find(e => e.Key == "iron-plate");
            Assert.NotNull(ironPlate);
            Assert.Equal("鉄板", ironPlate.Value);
            Assert.Equal("; Item Comment", ironPlate.Comment);

            Assert.Single(result.HeaderComments);
            Assert.Equal("; Header Comment", result.HeaderComments[0]);
        }

        [Fact]
        public void Write_ShouldPreserveStructure()
        {
            // Arrange
            var parser = new CfgParser();
            var cfgFile = new CfgFile
            {
                SectionOrder = new List<string> { "item-name" },
                HeaderComments = new List<string> { "; Header" },
                Entries = new List<CfgEntry>
                {
                    new CfgEntry { Section = "item-name", Key = "key1", Value = "val1", Comment = "; Entry Comment" }
                }
            };
            using var stream = new MemoryStream();

            // Act
            parser.Write(cfgFile, stream);
            var output = Encoding.UTF8.GetString(stream.ToArray());

            // Assert
            Assert.Contains("; Header", output);
            Assert.Contains("[item-name]", output);
            Assert.Contains("; Entry Comment", output);
            Assert.Contains("key1=val1", output);
        }
    }
}
