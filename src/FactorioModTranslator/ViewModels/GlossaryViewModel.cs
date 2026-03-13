using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using FactorioModTranslator.Models;
using FactorioModTranslator.Services;
using System.Collections.ObjectModel;
using System.Windows.Input;

namespace FactorioModTranslator.ViewModels
{
    public partial class GlossaryViewModel : ObservableObject
    {
        private readonly GlossaryService _glossaryService;

        [ObservableProperty] private ObservableCollection<GlossaryEntry> _entries = new();
        [ObservableProperty] private string _newSourceTerm = string.Empty;
        [ObservableProperty] private string _newTargetTerm = string.Empty;

        public ICommand AddCommand { get; }
        public ICommand DeleteCommand { get; }

        public GlossaryViewModel(GlossaryService glossaryService)
        {
            _glossaryService = glossaryService;
            Entries = new ObservableCollection<GlossaryEntry>(_glossaryService.GetAllEntries());

            AddCommand = new RelayCommand(AddEntry);
            DeleteCommand = new RelayCommand<GlossaryEntry>(DeleteEntry);
        }

        private void AddEntry()
        {
            if (string.IsNullOrWhiteSpace(NewSourceTerm) || string.IsNullOrWhiteSpace(NewTargetTerm))
            {
                Log.Warn("AddEntry aborted: Source or Target term is empty.");
                return;
            }

            Log.Info($"Adding glossary entry: {NewSourceTerm} -> {NewTargetTerm}");
            var entry = new GlossaryEntry
            {
                SourceTerm = NewSourceTerm,
                TargetTerm = NewTargetTerm,
                SourceLang = "en", // Default for now
                TargetLang = "ja"
            };

            _glossaryService.AddEntry(entry);
            Entries.Add(entry);
            
            Log.Debug("Glossary entry added and saved.");
            NewSourceTerm = string.Empty;
            NewTargetTerm = string.Empty;
        }

        private void DeleteEntry(GlossaryEntry? entry)
        {
            if (entry == null)
            {
                Log.Warn("DeleteEntry called with null entry.");
                return;
            }

            Log.Info($"Deleting glossary entry: {entry.SourceTerm}");
            _glossaryService.RemoveEntry(entry.SourceTerm);
            Entries.Remove(entry);
            Log.Debug("Glossary entry removed.");
        }
    }
}
