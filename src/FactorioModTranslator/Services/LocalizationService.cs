using System;
using System.Collections.Generic;
using System.ComponentModel;
using System.Globalization;
using System.Runtime.CompilerServices;
using System.Windows;

namespace FactorioModTranslator.Services
{
    public class LocalizationService : INotifyPropertyChanged
    {
        private static readonly LocalizationService _instance = new();
        public static LocalizationService Instance => _instance;

        private CultureInfo _currentCulture = CultureInfo.CurrentCulture;

        public CultureInfo CurrentCulture
        {
            get => _currentCulture;
            set
            {
                if (_currentCulture != value)
                {
                    _currentCulture = value;
                    OnPropertyChanged();
                    OnPropertyChanged("Item[]");
                }
            }
        }

        public string this[string key]
        {
            get
            {
                // In a real app, this would use Resource files (.resx) or a dictionary.
                // For simplicity, we can use a hardcoded map or load from JSON.
                return GetString(key);
            }
        }

        private string GetString(string key)
        {
            bool isJa = _currentCulture.TwoLetterISOLanguageName == "ja";
            
            return key switch
            {
                "AppTitle" => isJa ? "Factorio Mod 自動翻訳ツール" : "Factorio Mod Auto-Translator",
                "SelectMod" => isJa ? "Modを選択" : "Select Mod",
                "Translate" => isJa ? "翻訳実行" : "Translate",
                "Settings" => isJa ? "設定" : "Settings",
                "Glossary" => isJa ? "用語集" : "Glossary",
                "History" => isJa ? "履歴" : "History",
                "Mode" => isJa ? "モード" : "Mode",
                "Engine" => isJa ? "エンジン" : "Engine",
                "TranslationMode_NewTranslation" => isJa ? "新規翻訳" : "New Translation",
                "TranslationMode_DiffTranslation" => isJa ? "差分翻訳" : "Diff Translation",
                "TranslationMode_OverwriteUpdate" => isJa ? "上書き更新" : "Overwrite Update",
                "TranslationMode_ManualEdit" => isJa ? "手動編集" : "Manual Edit",
                _ => key
            };
        }

        public event PropertyChangedEventHandler? PropertyChanged;
        protected void OnPropertyChanged([CallerMemberName] string? name = null) => PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(name));
    }
}
