using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using FactorioModTranslator.Models;
using FactorioModTranslator.Services;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using System.Windows.Input;

namespace FactorioModTranslator.ViewModels
{
    public partial class TranslationPreviewViewModel : ObservableObject
    {
        private readonly CfgParser _cfgParser;
        private ModInfo? _loadedMod;
        private string _targetLang = string.Empty;

        [ObservableProperty] private ObservableCollection<TranslationItem> _items = new();
        [ObservableProperty] private string _filterText = string.Empty;
        [ObservableProperty] private bool _isSaving;

        public IAsyncRelayCommand SaveCommand { get; }

        public TranslationPreviewViewModel(CfgParser cfgParser)
        {
            _cfgParser = cfgParser;
            SaveCommand = new AsyncRelayCommand(SaveAsync);
        }

        public void LoadItems(List<TranslationItem> items, ModInfo mod, string targetLang)
        {
            Items = new ObservableCollection<TranslationItem>(items);
            _loadedMod = mod;
            _targetLang = targetLang;
        }

        private async Task SaveAsync()
        {
            if (_loadedMod == null || string.IsNullOrEmpty(_targetLang)) return;

            IsSaving = true;
            try
            {
                // Group items by their original cfg file structure or just create a new one
                // For simplicity, we create one cfg file per original cfg file but in the target locale folder
                var groupedItems = Items.GroupBy(i => "translation.cfg"); // Simplified: one file per mod

                string localePath = Path.Combine(_loadedMod.SourcePath, "locale", _targetLang);
                if (!Directory.Exists(localePath)) Directory.CreateDirectory(localePath);

                string filePath = Path.Combine(localePath, "translation.cfg");

                var cfgFile = new CfgFile
                {
                    LanguageCode = _targetLang,
                    FilePath = filePath
                };

                foreach (var item in Items)
                {
                    if (!cfgFile.SectionOrder.Contains(item.Section))
                        cfgFile.SectionOrder.Add(item.Section);

                    cfgFile.Entries.Add(new CfgEntry
                    {
                        Section = item.Section,
                        Key = item.Key,
                        Value = item.TranslatedText
                    });
                }

                using var stream = File.Create(filePath);
                _cfgParser.Write(cfgFile, stream);

                // Success message or event
            }
            finally
            {
                IsSaving = false;
            }
        }
    }
}
