using System;
using System.Net;
using System.Text.RegularExpressions;

namespace FactorioModTranslator.Services
{
    public static class FactorioStringFormatter
    {
        // Regex to match:
        // 1. __(?:(?:(?!__).)+?__)+ (Variables like __1__, __ITEM__iron-plate__, __plural_for_parameter_1_{...}__)
        //    Matches __ followed by one or more groups of (non-double-underscore text + __).
        // 2. \[.*?\] (Rich text like [item=iron-plate], [color=red]...[/color])
        // 3. \\n (Escaped newline characters)
        private static readonly Regex ProtectionRegex = new Regex(@"(__ (?:(?:(?!__).)+?__)+)|(\[.*?\])|(\\n)", RegexOptions.Compiled | RegexOptions.IgnorePatternWhitespace);

        public static string WrapTags(string text)
        {
            if (string.IsNullOrEmpty(text)) return text;

            // 1. HtmlEncode to make it safe for XML TagHandling in DeepL
            string encoded = WebUtility.HtmlEncode(text);

            // 2. Wrap protected parts in <keep> tags
            return ProtectionRegex.Replace(encoded, match => $"<keep>{match.Value}</keep>");
        }

        public static string UnwrapTags(string translatedText)
        {
            if (string.IsNullOrEmpty(translatedText)) return translatedText;

            // 1. Remove <keep> tags
            string unwrapped = translatedText.Replace("<keep>", "").Replace("</keep>", "");

            // 2. HtmlDecode to restore original symbols correctly
            return WebUtility.HtmlDecode(unwrapped);
        }
    }
}
