using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using FactorioModTranslator.Models;
using FactorioModTranslator.Services;
using Microsoft.Win32;
using System;
using System.Collections.ObjectModel;
using System.IO;
using System.Threading.Tasks;
using System.Windows.Input;

namespace FactorioModTranslator.ViewModels
{
    public partial class ModSelectionViewModel : ObservableObject
    {
        private readonly ModLoader _modLoader;
        private readonly TranslationOrchestrator _orchestrator;
        private readonly SettingsService _settings;

        [ObservableProperty] private ModInfo? _loadedMod;
        [ObservableProperty] private TranslationMode _selectedMode = TranslationMode.NewTranslation;
        [ObservableProperty] private string _sourceLanguage = "en";
        [ObservableProperty] private string _targetLanguage = "ja";
        [ObservableProperty] private bool _isTranslating;
        [ObservableProperty] private double _progress;
        [ObservableProperty] private string _statusMessage = string.Empty;

        public IAsyncRelayCommand SelectFolderCommand { get; }
        public IAsyncRelayCommand SelectZipCommand { get; }
        public IAsyncRelayCommand StartTranslationCommand { get; }

        public Action<List<TranslationItem>, ModInfo, string>? OnTranslationComplete { get; set; }

        public ModSelectionViewModel(ModLoader modLoader, TranslationOrchestrator orchestrator, SettingsService settings)
        {
            _modLoader = modLoader;
            _orchestrator = orchestrator;
            _settings = settings;

            SelectFolderCommand = new AsyncRelayCommand(SelectFolderAsync);
            SelectZipCommand = new AsyncRelayCommand(SelectZipAsync);
            StartTranslationCommand = new AsyncRelayCommand(StartTranslationAsync, () => LoadedMod != null && !IsTranslating);
        }

        private async Task SelectFolderAsync()
        {
            var dialog = new OpenFolderDialog(); // Requires .NET 8.0 WPF or later
            if (dialog.ShowDialog() == true)
            {
                try
                {
                    LoadedMod = _modLoader.LoadFromFolder(dialog.FolderName);
                    StatusMessage = $"Loaded: {LoadedMod.Title} ({LoadedMod.Version})";
                    StartTranslationCommand.NotifyCanExecuteChanged();
                }
                catch (Exception ex)
                {
                    StatusMessage = $"Error: {ex.Message}";
                }
            }
        }

        private async Task SelectZipAsync()
        {
            var dialog = new OpenFileDialog { Filter = "Factorio Mod ZIP (*.zip)|*.zip" };
            if (dialog.ShowDialog() == true)
            {
                try
                {
                    LoadedMod = _modLoader.LoadFromZip(dialog.FileName);
                    StatusMessage = $"Loaded ZIP: {LoadedMod.Title}";
                    StartTranslationCommand.NotifyCanExecuteChanged();
                }
                catch (Exception ex)
                {
                    StatusMessage = $"Error: {ex.Message}";
                }
            }
        }

        private async Task StartTranslationAsync()
        {
            if (LoadedMod == null) return;

            IsTranslating = true;
            Progress = 0;
            StatusMessage = "Translating...";

            try
            {
                var progressReporter = new Progress<double>(p => Progress = p * 100);
                var results = await _orchestrator.ExecuteTranslationAsync(
                    LoadedMod, SelectedMode, SourceLanguage, TargetLanguage, progressReporter);

                StatusMessage = $"Translation complete! {results.Count} items processed.";
                OnTranslationComplete?.Invoke(results, LoadedMod, TargetLanguage);
            }
            catch (Exception ex)
            {
                StatusMessage = $"Translation failed: {ex.Message}";
            }
            finally
            {
                IsTranslating = false;
            }
        }
    }
}
