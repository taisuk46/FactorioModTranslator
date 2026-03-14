using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using Google.Cloud.Translation.V2;
using FactorioModTranslator.Models;

namespace FactorioModTranslator.Services
{
    public class GoogleTranslationEngine : ITranslationEngine
    {
        private TranslationClient? _client;
        private readonly string _apiKey;

        public string Name => "Google Translate";

        public GoogleTranslationEngine(string apiKey)
        {
            _apiKey = apiKey;
            if (!string.IsNullOrWhiteSpace(_apiKey))
            {
                _client = TranslationClient.CreateFromApiKey(_apiKey);
            }
        }

        public async Task<string> TranslateAsync(string text, string sourceLang, string targetLang)
        {
            Log.Info($"Google.TranslateAsync started: sourceLang={sourceLang}, targetLang={targetLang}, textLength={text.Length}");
            if (_client == null)
            {
                if (string.IsNullOrWhiteSpace(_apiKey))
                {
                    Log.Warn("Google API Key is missing.");
                    throw new InvalidOperationException("Google API Key is not set. Please go to Settings.");
                }
                Log.Debug("Creating Google TranslationClient.");
                _client = TranslationClient.CreateFromApiKey(_apiKey);
            }

            var result = await _client.TranslateTextAsync(text, targetLang, sourceLang);
            Log.Info($"Google.TranslateAsync completed. ResultLength={result.TranslatedText.Length}");
            return result.TranslatedText;
        }

        public async Task<List<string>> TranslateBatchAsync(IEnumerable<string> texts, string sourceLang, string targetLang)
        {
            var textList = texts.ToList();
            Log.Info($"Google.TranslateBatchAsync started: sourceLang={sourceLang}, targetLang={targetLang}, count={textList.Count}");
            if (_client == null)
            {
                if (string.IsNullOrWhiteSpace(_apiKey))
                {
                    Log.Warn("Google API Key is missing for batch.");
                    throw new InvalidOperationException("Google API Key is not set. Please go to Settings.");
                }
                Log.Debug("Creating Google TranslationClient for batch.");
                _client = TranslationClient.CreateFromApiKey(_apiKey);
            }

            var results = await _client.TranslateHtmlAsync(textList, targetLang, sourceLang); 
            Log.Info("Google.TranslateBatchAsync completed.");
            return results.Select(r => r.TranslatedText).ToList();
        }

        public void SetGlossary(IEnumerable<GlossaryEntry> glossary)
        {
            // Google Translate V2 API Key doesn't support glossaries easily like Advanced (V3).
            // We can implement client-side matching.
        }
    }
}
