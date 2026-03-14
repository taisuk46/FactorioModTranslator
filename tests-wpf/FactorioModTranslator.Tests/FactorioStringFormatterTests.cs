using Xunit;
using FactorioModTranslator.Services;

namespace FactorioModTranslator.Tests
{
    public class FactorioStringFormatterTests
    {
        [Theory]
        [InlineData("Normal text", "Normal text")]
        [InlineData("Variable __1__", "Variable <keep>__1__</keep>")]
        [InlineData("Item __ITEM__iron-plate__", "Item <keep>__ITEM__iron-plate__</keep>")]
        [InlineData("Plural __plural_for_parameter_1_{1=hour|rest=hours}__", "Plural <keep>__plural_for_parameter_1_{1=hour|rest=hours}__</keep>")]
        [InlineData("Rich [item=iron-plate]", "Rich <keep>[item=iron-plate]</keep>")]
        [InlineData("Color [color=red]Text[/color]", "Color <keep>[color=red]</keep>Text<keep>[/color]</keep>")]
        [InlineData("Newline\\nNext line", "Newline<keep>\\n</keep>Next line")]
        [InlineData("HTML <special>", "HTML &lt;special&gt;")]
        [InlineData("Mixed __1__ and [item=copper-plate]", "Mixed <keep>__1__</keep> and <keep>[item=copper-plate]</keep>")]
        public void WrapTags_ShouldProtectSpecialSequences(string input, string expected)
        {
            // Act
            string result = FactorioStringFormatter.WrapTags(input);

            // Assert
            Assert.Equal(expected, result);
        }

        [Theory]
        [InlineData("Normal text", "Normal text")]
        [InlineData("Variable <keep>__1__</keep>", "Variable __1__")]
        [InlineData("Item <keep>__ITEM__iron-plate__</keep>", "Item __ITEM__iron-plate__")]
        [InlineData("Rich <keep>[item=iron-plate]</keep>", "Rich [item=iron-plate]")]
        [InlineData("Color <keep>[color=red]</keep>Text<keep>[/color]</keep>", "Color [color=red]Text[/color]")]
        [InlineData("Newline<keep>\\n</keep>Next line", "Newline\\nNext line")]
        [InlineData("HTML &lt;special&gt;", "HTML <special>")]
        public void UnwrapTags_ShouldRestoreSpecialSequences(string input, string expected)
        {
            // Act
            string result = FactorioStringFormatter.UnwrapTags(input);

            // Assert
            Assert.Equal(expected, result);
        }

        [Fact]
        public void RoundTrip_ShouldPreserveOriginalContent()
        {
            // Arrange
            string original = "Test: __1__ [item=iron-plate] \\n <symbol>";

            // Act
            string wrapped = FactorioStringFormatter.WrapTags(original);
            string unwrapped = FactorioStringFormatter.UnwrapTags(wrapped);

            // Assert
            Assert.Equal(original, unwrapped);
        }
    }
}
