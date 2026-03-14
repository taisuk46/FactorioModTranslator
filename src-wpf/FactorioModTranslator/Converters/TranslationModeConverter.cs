using System;
using System.Globalization;
using System.Windows.Data;
using FactorioModTranslator.Models;
using FactorioModTranslator.Services;

namespace FactorioModTranslator.Converters
{
    public class TranslationModeConverter : IValueConverter
    {
        public object Convert(object value, Type targetType, object parameter, CultureInfo culture)
        {
            if (value is TranslationMode mode)
            {
                string key = $"TranslationMode_{mode}";
                return LocalizationService.Instance[key];
            }
            return value;
        }

        public object ConvertBack(object value, Type targetType, object parameter, CultureInfo culture)
        {
            throw new NotImplementedException();
        }
    }
}
