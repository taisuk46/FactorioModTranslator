using System;
using System.Diagnostics;
using System.IO;
using System.Runtime.CompilerServices;

namespace FactorioModTranslator.Services
{
    public class LogService
    {
        private static LogService? _instance;
        public static LogService Instance => _instance ??= new LogService();

        private LogService() { }

        public void Initialize()
        {
            string appData = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "FactorioModTranslator");
            string logDir = Path.Combine(appData, "logs");
            if (!Directory.Exists(logDir)) Directory.CreateDirectory(logDir);

            string logPath = Path.Combine(logDir, $"log_{DateTime.Now:yyyyMMdd}.txt");
            
            var listener = new TextWriterTraceListener(logPath);
            Trace.Listeners.Add(listener);
            Trace.AutoFlush = true;

            Info("Logging system initialized.");
        }

        public void Info(string message, [CallerFilePath] string filePath = "", [CallerLineNumber] int lineNumber = 0)
            => WriteLog("INFO", message, filePath, lineNumber);

        public void Warn(string message, [CallerFilePath] string filePath = "", [CallerLineNumber] int lineNumber = 0)
            => WriteLog("WARN", message, filePath, lineNumber);

        public void Error(string message, [CallerFilePath] string filePath = "", [CallerLineNumber] int lineNumber = 0)
            => WriteLog("ERROR", message, filePath, lineNumber);

        public void Error(string message, Exception ex, [CallerFilePath] string filePath = "", [CallerLineNumber] int lineNumber = 0)
            => WriteLog("ERROR", $"{message} Exception: {ex}", filePath, lineNumber);

        public void Debug(string message, [CallerFilePath] string filePath = "", [CallerLineNumber] int lineNumber = 0)
            => WriteLog("DEBUG", message, filePath, lineNumber);

        private void WriteLog(string level, string message, string filePath, int lineNumber)
        {
            string fileName = Path.GetFileName(filePath);
            string timestamp = DateTime.Now.ToString("yyyy-MM-dd HH:mm:ss.fff");
            string logEntry = $"[{timestamp}] [{level}] [{fileName}:{lineNumber}] {message}";
            
            Trace.WriteLine(logEntry);
            Console.WriteLine(logEntry); // Also output to console for debug
        }
    }
}
