using System;
using System.Runtime.CompilerServices;
using FactorioModTranslator.Services;

namespace FactorioModTranslator
{
    public static class Log
    {
        public static void Info(string message, [CallerFilePath] string filePath = "", [CallerLineNumber] int lineNumber = 0)
            => LogService.Instance.Info(message, filePath, lineNumber);

        public static void Warn(string message, [CallerFilePath] string filePath = "", [CallerLineNumber] int lineNumber = 0)
            => LogService.Instance.Warn(message, filePath, lineNumber);

        public static void Error(string message, [CallerFilePath] string filePath = "", [CallerLineNumber] int lineNumber = 0)
            => LogService.Instance.Error(message, filePath, lineNumber);

        public static void Error(string message, Exception ex, [CallerFilePath] string filePath = "", [CallerLineNumber] int lineNumber = 0)
            => LogService.Instance.Error(message, ex, filePath, lineNumber);

        public static void Debug(string message, [CallerFilePath] string filePath = "", [CallerLineNumber] int lineNumber = 0)
            => LogService.Instance.Debug(message, filePath, lineNumber);
    }
}
