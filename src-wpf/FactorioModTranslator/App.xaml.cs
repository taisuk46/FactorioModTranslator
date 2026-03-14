using System.Windows;
using FactorioModTranslator.Services;

namespace FactorioModTranslator
{
    public partial class App : Application
    {
        public App()
        {
            LogService.Instance.Initialize();

            AppDomain.CurrentDomain.UnhandledException += (s, e) => {
                var message = "Fatal Unhandled Exception";
                Log.Error(message, (Exception)e.ExceptionObject);
                MessageBox.Show(message + ": " + e.ExceptionObject.ToString(), "Fatal Error", MessageBoxButton.OK, MessageBoxImage.Error);
            };

            this.DispatcherUnhandledException += (s, e) => {
                e.Handled = true;
                var message = "Dispatcher Unhandled Exception";
                Log.Error(message, e.Exception);
                MessageBox.Show(message + ": " + e.Exception.ToString(), "Dispatcher Error", MessageBoxButton.OK, MessageBoxImage.Error);
            };
        }
    }
}
