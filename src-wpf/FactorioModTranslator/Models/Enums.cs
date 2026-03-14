namespace FactorioModTranslator.Models
{
    public enum ModSourceType
    {
        Folder,
        Zip
    }

    public enum TranslationMode
    {
        NewTranslation,
        DiffTranslation,
        OverwriteUpdate,
        ManualEdit
    }

    public enum TranslationSource
    {
        API,
        VanillaKeyMatch,
        VanillaTextMatch,
        Manual,
        History,
        Glossary
    }
}
