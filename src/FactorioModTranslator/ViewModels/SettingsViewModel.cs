using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using FactorioModTranslator.Services;
using Microsoft.Win32;
using System.Threading.Tasks;
using System.Windows.Input;

namespace FactorioModTranslator.ViewModels
{
    public partial class SettingsViewModel : ObservableObject
    {
        private readonly SettingsService _settingsService;

        [ObservableProperty] private string _deepLApiKey = string.Empty;
        [ObservableProperty] private string _googleApiKey = string.Empty;
        [ObservableProperty] private string _factorioPath = string.Empty;
        [ObservableProperty] private string _selectedEngine = "DeepL";

        public ICommand SaveCommand { get; }
        public ICommand BrowseFactorioPathCommand { get; }

        public SettingsViewModel(SettingsService settingsService)
        {
            _settingsService = settingsService;

            DeepLApiKey = _settingsService.LoadApiKey("DeepL") ?? string.Empty;
            GoogleApiKey = _settingsService.LoadApiKey("Google") ?? string.Empty;
            FactorioPath = _settingsService.Current.FactorioPath;
            SelectedEngine = _settingsService.Current.SelectedEngine;

            SaveCommand = new RelayCommand(Save);
            BrowseFactorioPathCommand = new RelayCommand(BrowseFactorioPath);
        }

        private void BrowseFactorioPath()
        {
            Log.Info("Browsing for Factorio path.");
            var dialog = new OpenFolderDialog { Title = "Select Factorio Installation Folder" };
            if (dialog.ShowDialog() == true)
            {
                FactorioPath = dialog.FolderName;
                Log.Info($"Factorio path selected: {FactorioPath}");
            }
            else
            {
                Log.Info("Factorio path selection cancelled.");
            }
        }
 
        private void Save()
        {
            Log.Info($"Saving settings. Engine={SelectedEngine}, Path={FactorioPath}");
            var settings = _settingsService.Current;
            settings.FactorioPath = FactorioPath;
            settings.SelectedEngine = SelectedEngine;
            
            _settingsService.SaveSettings(settings);
            
            if (!string.IsNullOrEmpty(DeepLApiKey))
            {
                Log.Debug("Saving DeepL API Key.");
                _settingsService.SaveApiKey("DeepL", DeepLApiKey);
            }
            
            if (!string.IsNullOrEmpty(GoogleApiKey))
            {
                Log.Debug("Saving Google API Key.");
                _settingsService.SaveApiKey("Google", GoogleApiKey);
            }

            Log.Info("Settings saved successfully.");
        }
    }
}
