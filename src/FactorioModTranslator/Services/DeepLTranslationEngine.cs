using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using DeepL;
using FactorioModTranslator.Models;

namespace FactorioModTranslator.Services
{
    public class DeepLTranslationEngine : ITranslationEngine
    {
        private Translator? _translator;
        private string _apiKey = string.Empty;
        private readonly SettingsService _settings;

        public string Name => "DeepL";

        public DeepLTranslationEngine(SettingsService settings)
        {
            _settings = settings;
        }

        public async Task<string> TranslateAsync(string text, string sourceLang, string targetLang)
        {
            var currentKey = _settings.LoadApiKey("DeepL") ?? string.Empty;
            if (_translator == null || _apiKey != currentKey)
            {
                _apiKey = currentKey;
                if (string.IsNullOrWhiteSpace(_apiKey)) 
                    throw new InvalidOperationException("DeepL API Key is not set. Please go to Settings.");
                
                _translator = new Translator(_apiKey);
            }

            var options = new TextTranslateOptions();
            var result = await _translator.TranslateTextAsync(text, MapLang(sourceLang), MapLang(targetLang), options);
            return result.Text;
        }

        public async Task<List<string>> TranslateBatchAsync(IEnumerable<string> texts, string sourceLang, string targetLang)
        {
            var currentKey = _settings.LoadApiKey("DeepL") ?? string.Empty;
            if (_translator == null || _apiKey != currentKey)
            {
                _apiKey = currentKey;
                if (string.IsNullOrWhiteSpace(_apiKey)) 
                    throw new InvalidOperationException("DeepL API Key is not set. Please go to Settings.");
                
                _translator = new Translator(_apiKey);
            }

            var options = new TextTranslateOptions();
            var results = await _translator.TranslateTextAsync(texts, MapLang(sourceLang), MapLang(targetLang), options);
            return results.Select(r => r.Text).ToList();
        }

        public void SetGlossary(IEnumerable<GlossaryEntry> glossary)
        {
            // Future implementation
        }

        private string MapLang(string lang)
        {
            lang = lang.ToUpper();
            if (lang == "EN") return "EN-US";
            if (lang == "JA") return "JA";
            return lang;
        }
    }
}
