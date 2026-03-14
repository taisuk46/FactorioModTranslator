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
            Log.Info($"TranslateAsync started: sourceLang={sourceLang}, targetLang={targetLang}, textLength={text.Length}");
            
            var currentKey = _settings.LoadApiKey("DeepL") ?? string.Empty;
            if (_translator == null || _apiKey != currentKey)
            {
                Log.Info("Initializing or updating DeepL Translator instance.");
                _apiKey = currentKey;
                if (string.IsNullOrWhiteSpace(_apiKey)) 
                {
                    Log.Warn("DeepL API Key is missing.");
                    throw new InvalidOperationException("DeepL API Key is not set. Please go to Settings.");
                }
                
                _translator = new Translator(_apiKey);
                Log.Debug("DeepL Translator instance created successfully.");
            }

            var wrappedText = FactorioStringFormatter.WrapTags(text);
            var options = new TextTranslateOptions
            {
                TagHandling = "xml",
                IgnoreTags = { "keep" }
            };

            var result = await _translator.TranslateTextAsync(wrappedText, MapLang(sourceLang, false), MapLang(targetLang, true), options);
            
            var finalResult = FactorioStringFormatter.UnwrapTags(result.Text);
            Log.Info($"TranslateAsync completed. ResultLength={finalResult.Length}");
            return finalResult;
        }

        public async Task<List<string>> TranslateBatchAsync(IEnumerable<string> texts, string sourceLang, string targetLang)
        {
            var textList = texts.ToList();
            Log.Info($"TranslateBatchAsync started: sourceLang={sourceLang}, targetLang={targetLang}, count={textList.Count}");

            var currentKey = _settings.LoadApiKey("DeepL") ?? string.Empty;
            if (_translator == null || _apiKey != currentKey)
            {
                Log.Info("Initializing or updating DeepL Translator instance for batch.");
                _apiKey = currentKey;
                if (string.IsNullOrWhiteSpace(_apiKey)) 
                {
                    Log.Warn("DeepL API Key is missing for batch.");
                    throw new InvalidOperationException("DeepL API Key is not set. Please go to Settings.");
                }
                
                _translator = new Translator(_apiKey);
                Log.Debug("DeepL Translator instance created for batch successfully.");
            }

            var wrappedTexts = textList.Select(FactorioStringFormatter.WrapTags).ToList();
            var options = new TextTranslateOptions
            {
                TagHandling = "xml",
                IgnoreTags = { "keep" }
            };

            var results = await _translator.TranslateTextAsync(wrappedTexts, MapLang(sourceLang, false), MapLang(targetLang, true), options);
            
            Log.Info("TranslateBatchAsync completed.");
            return results.Select(r => FactorioStringFormatter.UnwrapTags(r.Text)).ToList();
        }

        public void SetGlossary(IEnumerable<GlossaryEntry> glossary)
        {
            // Future implementation
        }

        private string MapLang(string lang, bool isTarget)
        {
            lang = lang.ToUpper();
            if (lang == "EN")
            {
                return isTarget ? "EN-US" : "EN";
            }
            return lang;
        }
    }
}
