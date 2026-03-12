using System.Collections.Generic;
using System.Threading.Tasks;

namespace FactorioModTranslator.Services
{
    public interface ITranslationEngine
    {
        string Name { get; }
        
        /// <summary>
        /// Translates a single text.
        /// </summary>
        Task<string> TranslateAsync(string text, string sourceLang, string targetLang);

        /// <summary>
        /// Translates a batch of texts for efficiency.
        /// </summary>
        Task<List<string>> TranslateBatchAsync(IEnumerable<string> texts, string sourceLang, string targetLang);

        /// <summary>
        /// Sets the glossary to use for translation.
        /// </summary>
        void SetGlossary(IEnumerable<Models.GlossaryEntry> glossary);
    }
}
