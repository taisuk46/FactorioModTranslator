using System.Windows;
using FactorioModTranslator.Services;
using FactorioModTranslator.ViewModels;

namespace FactorioModTranslator;

public partial class MainWindow : Window
{
    public MainWindow()
    {
        InitializeComponent();
        
        var cfgParser = new CfgParser();
        var modLoader = new ModLoader(cfgParser);
        var glossaryService = new GlossaryService();
        var historyService = new TranslationHistoryService();
        var settingsService = new SettingsService();
        var vanillaService = new VanillaTranslationService(cfgParser);

        var apiKey = settingsService.LoadApiKey("DeepL") ?? "";
        var engine = new DeepLTranslationEngine(apiKey);

        var orchestrator = new TranslationOrchestrator(engine, vanillaService, glossaryService, historyService);

        var modSelectionVM = new ModSelectionViewModel(modLoader, orchestrator, settingsService);
        var previewVM = new TranslationPreviewViewModel(cfgParser);
        var settingsVM = new SettingsViewModel(settingsService);
        var glossaryVM = new GlossaryViewModel(glossaryService);

        var mainVM = new MainViewModel(modSelectionVM, previewVM, settingsVM, glossaryVM);

        this.DataContext = mainVM;
    }
}