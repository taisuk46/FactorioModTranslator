using System.Windows;

namespace FactorioModTranslator
{
    public partial class App : Application
    {
        public App()
        {
            AppDomain.CurrentDomain.UnhandledException += (s, e) => {
                MessageBox.Show("Fatal Error: " + e.ExceptionObject.ToString(), "Fatal Error", MessageBoxButton.OK, MessageBoxImage.Error);
            };

            this.DispatcherUnhandledException += (s, e) => {
                e.Handled = true;
                MessageBox.Show("Dispatcher Error: " + e.Exception.ToString(), "Dispatcher Error", MessageBoxButton.OK, MessageBoxImage.Error);
            };
        }
    }
}
