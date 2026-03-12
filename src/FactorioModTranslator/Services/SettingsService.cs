using System;
using System.IO;
using System.Security.Cryptography;
using System.Text;
using System.Text.Json;

namespace FactorioModTranslator.Services
{
    public class AppSettings
    {
        public string SelectedEngine { get; set; } = "DeepL";
        public string FactorioPath { get; set; } = @"C:\Program Files\Factorio";
        public string UILanguage { get; set; } = "ja";
        public string LastModPath { get; set; } = string.Empty;
    }

    public class SettingsService
    {
        private readonly string _settingsPath;
        private readonly string _apiKeyPath;
        private AppSettings _current;

        public AppSettings Current => _current;

        public SettingsService()
        {
            string appData = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "FactorioModTranslator");
            _settingsPath = Path.Combine(appData, "settings.json");
            _apiKeyPath = Path.Combine(appData, "keys.dat");
            _current = LoadSettings();
        }

        private AppSettings LoadSettings()
        {
            if (File.Exists(_settingsPath))
            {
                try
                {
                    return JsonSerializer.Deserialize<AppSettings>(File.ReadAllText(_settingsPath)) ?? new AppSettings();
                }
                catch { }
            }
            return new AppSettings();
        }

        public void SaveSettings(AppSettings settings)
        {
            _current = settings;
            string directory = Path.GetDirectoryName(_settingsPath)!;
            if (!Directory.Exists(directory)) Directory.CreateDirectory(directory);

            File.WriteAllText(_settingsPath, JsonSerializer.Serialize(_current, new JsonSerializerOptions { WriteIndented = true }));
        }

        public void SaveApiKey(string engine, string key)
        {
            try
            {
                byte[] data = Encoding.UTF8.GetBytes($"{engine}:{key}");
                // In a real WPF app, use DPAPI. For now, we'll simulate or use a simple obfuscation 
                // if DPAPI assembly is tricky to reference directly in some environments, 
                // but standard .NET 8 on Windows has it in System.Security.Cryptography.ProtectedData.
                
                // Note: ProtectedData requires System.Security.Cryptography.ProtectedData NuGet package.
                // I'll add it if it's missing.
                byte[] encrypted = ProtectedData.Protect(data, null, DataProtectionScope.CurrentUser);
                
                string directory = Path.GetDirectoryName(_apiKeyPath)!;
                if (!Directory.Exists(directory)) Directory.CreateDirectory(directory);
                
                File.WriteAllBytes(_apiKeyPath, encrypted);
            }
            catch (Exception ex)
            {
                Console.WriteLine($"Error saving API key: {ex.Message}");
            }
        }

        public string? LoadApiKey(string engine)
        {
            if (!File.Exists(_apiKeyPath)) return null;
            try
            {
                byte[] encrypted = File.ReadAllBytes(_apiKeyPath);
                byte[] decrypted = ProtectedData.Unprotect(encrypted, null, DataProtectionScope.CurrentUser);
                string content = Encoding.UTF8.GetString(decrypted);
                
                // content format is "Engine:Key"
                if (content.StartsWith($"{engine}:"))
                {
                    return content.Substring(engine.Length + 1);
                }
            }
            catch { }
            return null;
        }
    }
}
