using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using FactorioModTranslator.Services;
using System.Collections.ObjectModel;
using System.Windows.Input;

namespace FactorioModTranslator.ViewModels
{
    public partial class MainViewModel : ObservableObject
    {
        private readonly LocalizationService _loc = LocalizationService.Instance;

        [ObservableProperty] private string _appTitle = string.Empty;
        [ObservableProperty] private int _selectedTabIndex;
        
        public ModSelectionViewModel ModSelection { get; }
        public TranslationPreviewViewModel TranslationPreview { get; }
        public SettingsViewModel Settings { get; }
        public GlossaryViewModel Glossary { get; }

        public ICommand SwitchLanguageCommand { get; }

        public MainViewModel(
            ModSelectionViewModel modSelection,
            TranslationPreviewViewModel translationPreview,
            SettingsViewModel settings,
            GlossaryViewModel glossary)
        {
            ModSelection = modSelection;
            TranslationPreview = translationPreview;
            Settings = settings;
            Glossary = glossary;

            AppTitle = _loc["AppTitle"];
            SwitchLanguageCommand = new RelayCommand(SwitchLanguage);
        }

        private void SwitchLanguage()
        {
            var current = _loc.CurrentCulture.TwoLetterISOLanguageName;
            var next = current == "ja" ? "en" : "ja";
            Log.Info($"Switching language from {current} to {next}");
            _loc.CurrentCulture = new System.Globalization.CultureInfo(next);
            AppTitle = _loc["AppTitle"];
        }
    }
}
