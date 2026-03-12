using System;
using System.Collections.Generic;
using System.IO;
using Microsoft.Data.Sqlite;
using FactorioModTranslator.Models;

namespace FactorioModTranslator.Services
{
    public class TranslationHistoryService
    {
        private readonly string _dbPath;
        private readonly string _connectionString;

        public TranslationHistoryService()
        {
            string appData = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "FactorioModTranslator");
            if (!Directory.Exists(appData)) Directory.CreateDirectory(appData);
            
            _dbPath = Path.Combine(appData, "history.db");
            _connectionString = $"Data Source={_dbPath}";
            
            InitializeDatabase();
        }

        private void InitializeDatabase()
        {
            using var connection = new SqliteConnection(_connectionString);
            connection.Open();

            var command = connection.CreateCommand();
            command.CommandText = @"
                CREATE TABLE IF NOT EXISTS translation_history (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    mod_name TEXT NOT NULL,
                    section TEXT NOT NULL,
                    key TEXT NOT NULL,
                    source_lang TEXT NOT NULL,
                    target_lang TEXT NOT NULL,
                    source_text TEXT NOT NULL,
                    translated_text TEXT NOT NULL,
                    engine TEXT NOT NULL,
                    translated_at TEXT NOT NULL,
                    UNIQUE(mod_name, section, key, target_lang)
                );
            ";
            command.ExecuteNonQuery();
        }

        public void SaveRecord(TranslationRecord record)
        {
            using var connection = new SqliteConnection(_connectionString);
            connection.Open();

            var command = connection.CreateCommand();
            command.CommandText = @"
                INSERT OR REPLACE INTO translation_history 
                (mod_name, section, key, source_lang, target_lang, source_text, translated_text, engine, translated_at)
                VALUES ($mod, $sec, $key, $slang, $tlang, $stext, $ttext, $eng, $at);
            ";
            command.Parameters.AddWithValue("$mod", record.ModName);
            command.Parameters.AddWithValue("$sec", record.Section);
            command.Parameters.AddWithValue("$key", record.Key);
            command.Parameters.AddWithValue("$slang", record.SourceLang);
            command.Parameters.AddWithValue("$tlang", record.TargetLang);
            command.Parameters.AddWithValue("$stext", record.SourceText);
            command.Parameters.AddWithValue("$ttext", record.TranslatedText);
            command.Parameters.AddWithValue("$eng", record.Engine);
            command.Parameters.AddWithValue("$at", record.TranslatedAt.ToString("o"));
            
            command.ExecuteNonQuery();
        }

        public string? GetPreviousTranslation(string modName, string section, string key, string targetLang)
        {
            using var connection = new SqliteConnection(_connectionString);
            connection.Open();

            var command = connection.CreateCommand();
            command.CommandText = "SELECT translated_text FROM translation_history WHERE mod_name = $mod AND section = $sec AND key = $key AND target_lang = $tlang LIMIT 1;";
            command.Parameters.AddWithValue("$mod", modName);
            command.Parameters.AddWithValue("$sec", section);
            command.Parameters.AddWithValue("$key", key);
            command.Parameters.AddWithValue("$tlang", targetLang);

            using var reader = command.ExecuteReader();
            if (reader.Read())
            {
                return reader.GetString(0);
            }
            return null;
        }

        public List<TranslationRecord> GetAllHistory()
        {
            var results = new List<TranslationRecord>();
            using var connection = new SqliteConnection(_connectionString);
            connection.Open();

            var command = connection.CreateCommand();
            command.CommandText = "SELECT * FROM translation_history ORDER BY translated_at DESC;";
            
            using var reader = command.ExecuteReader();
            while (reader.Read())
            {
                results.Add(new TranslationRecord
                {
                    Id = reader.GetInt32(0),
                    ModName = reader.GetString(1),
                    Section = reader.GetString(2),
                    Key = reader.GetString(3),
                    SourceLang = reader.GetString(4),
                    TargetLang = reader.GetString(5),
                    SourceText = reader.GetString(6),
                    TranslatedText = reader.GetString(7),
                    Engine = reader.GetString(8),
                    TranslatedAt = DateTime.Parse(reader.GetString(9))
                });
            }
            return results;
        }
    }
}
