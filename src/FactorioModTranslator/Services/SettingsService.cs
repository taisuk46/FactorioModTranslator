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

        public SettingsService(string? customSettingsPath = null, string? customKeyPath = null)
        {
            string appData = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "FactorioModTranslator");
            _settingsPath = customSettingsPath ?? Path.Combine(appData, "settings.json");
            _apiKeyPath = customKeyPath ?? Path.Combine(appData, "keys.dat");
            _current = LoadSettings();
        }

        private AppSettings LoadSettings()
        {
            if (File.Exists(_settingsPath))
            {
                try
                {
                    Log.Debug($"Loading settings from {_settingsPath}");
                    var json = File.ReadAllText(_settingsPath);
                    var settings = JsonSerializer.Deserialize<AppSettings>(json) ?? new AppSettings();
                    Log.Info("Settings loaded successfully.");
                    return settings;
                }
                catch (Exception ex)
                {
                    Log.Error("Failed to load settings file.", ex);
                }
            }
            else
            {
                Log.Info("Settings file not found, using defaults.");
            }
            return new AppSettings();
        }

        public void SaveSettings(AppSettings settings)
        {
            Log.Info($"Saving settings to {_settingsPath}");
            _current = settings;
            string directory = Path.GetDirectoryName(_settingsPath)!;
            if (!Directory.Exists(directory)) 
            {
                Log.Debug($"Creating directory: {directory}");
                Directory.CreateDirectory(directory);
            }

            var json = JsonSerializer.Serialize(_current, new JsonSerializerOptions { WriteIndented = true });
            File.WriteAllText(_settingsPath, json);
            Log.Info("Settings saved successfully.");
        }

        public void SaveApiKey(string engine, string key)
        {
            Log.Info($"SaveApiKey started for engine: {engine}");
            try
            {
                byte[] data = Encoding.UTF8.GetBytes($"{engine}:{key}");
                byte[] encrypted = ProtectedData.Protect(data, null, DataProtectionScope.CurrentUser);
                
                string directory = Path.GetDirectoryName(_apiKeyPath)!;
                if (!Directory.Exists(directory)) 
                {
                    Log.Debug($"Creating directory for keys: {directory}");
                    Directory.CreateDirectory(directory);
                }
                
                File.WriteAllBytes(_apiKeyPath, encrypted);
                Log.Info($"API key for {engine} saved successfully.");
            }
            catch (Exception ex)
            {
                Log.Error($"Error saving API key for {engine}", ex);
            }
        }

        public string? LoadApiKey(string engine)
        {
            if (!File.Exists(_apiKeyPath)) 
            {
                Log.Debug("API key file not found.");
                return null;
            }

            try
            {
                Log.Debug($"Loading API key for engine: {engine}");
                byte[] encrypted = File.ReadAllBytes(_apiKeyPath);
                byte[] decrypted = ProtectedData.Unprotect(encrypted, null, DataProtectionScope.CurrentUser);
                string content = Encoding.UTF8.GetString(decrypted);
                
                if (content.StartsWith($"{engine}:"))
                {
                    Log.Debug($"API key for {engine} loaded.");
                    return content.Substring(engine.Length + 1);
                }
                Log.Warn($"API key for {engine} not found in key file.");
            }
            catch (Exception ex)
            {
                Log.Error($"Error loading API key for {engine}", ex);
            }
            return null;
        }
    }
}
