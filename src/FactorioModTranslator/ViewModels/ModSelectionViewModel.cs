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
            Log.Info("SelectFolderAsync triggered.");
            var dialog = new OpenFolderDialog(); // Requires .NET 8.0 WPF or later
            if (dialog.ShowDialog() == true)
            {
                Log.Info($"Folder selected: {dialog.FolderName}");
                try
                {
                    LoadedMod = _modLoader.LoadFromFolder(dialog.FolderName);
                    StatusMessage = $"Loaded: {LoadedMod.Title} ({LoadedMod.Version})";
                    Log.Info($"Mod loaded successfully: {LoadedMod.Title}");
                    StartTranslationCommand.NotifyCanExecuteChanged();
                }
                catch (Exception ex)
                {
                    Log.Error("Failed to load mod from folder", ex);
                    StatusMessage = $"Error: {ex.Message}";
                }
            }
            else
            {
                Log.Info("Folder selection cancelled.");
            }
        }

        private async Task SelectZipAsync()
        {
            Log.Info("SelectZipAsync triggered.");
            var dialog = new OpenFileDialog { Filter = "Factorio Mod ZIP (*.zip)|*.zip" };
            if (dialog.ShowDialog() == true)
            {
                Log.Info($"ZIP selected: {dialog.FileName}");
                try
                {
                    LoadedMod = _modLoader.LoadFromZip(dialog.FileName);
                    StatusMessage = $"Loaded ZIP: {LoadedMod.Title}";
                    Log.Info($"Mod loaded successfully from ZIP: {LoadedMod.Title}");
                    StartTranslationCommand.NotifyCanExecuteChanged();
                }
                catch (Exception ex)
                {
                    Log.Error("Failed to load mod from ZIP", ex);
                    StatusMessage = $"Error: {ex.Message}";
                }
            }
            else
            {
                Log.Info("ZIP selection cancelled.");
            }
        }

        private async Task StartTranslationAsync()
        {
            if (LoadedMod == null)
            {
                Log.Warn("StartTranslationAsync called but LoadedMod is null.");
                return;
            }

            Log.Info($"Starting translation for {LoadedMod.Title}. Mode: {SelectedMode}, Source: {SourceLanguage}, Target: {TargetLanguage}");
            IsTranslating = true;
            Progress = 0;
            StatusMessage = "Translating...";

            try
            {
                var progressReporter = new Progress<double>(p => Progress = p * 100);
                var results = await _orchestrator.ExecuteTranslationAsync(
                    LoadedMod, SelectedMode, SourceLanguage, TargetLanguage, progressReporter);

                Log.Info($"Translation completed. Items: {results.Count}");
                StatusMessage = $"Translation complete! {results.Count} items processed.";
                OnTranslationComplete?.Invoke(results, LoadedMod, TargetLanguage);
            }
            catch (Exception ex)
            {
                Log.Error("Translation execution failed", ex);
                StatusMessage = $"Translation failed: {ex.Message}";
            }
            finally
            {
                IsTranslating = false;
            }
        }
    }
}
