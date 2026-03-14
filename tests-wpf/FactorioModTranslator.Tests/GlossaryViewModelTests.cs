using Xunit;
using Moq;
using FactorioModTranslator.ViewModels;
using FactorioModTranslator.Services;
using FactorioModTranslator.Models;
using System.Collections.Generic;
using System.Linq;

namespace FactorioModTranslator.Tests
{
    public class GlossaryViewModelTests
    {
        [Fact]
        public void DeleteCommand_WithInvalidParameter_ShouldNotThrow()
        {
            // Arrange
            string tempDb = System.IO.Path.Combine(System.IO.Path.GetTempPath(), System.Guid.NewGuid().ToString() + ".json");
            var service = new GlossaryService(tempDb);
            var viewModel = new GlossaryViewModel(service);
            
            // Act & Assert
            var exception = Record.Exception(() => viewModel.DeleteCommand.Execute(new object()));
            
            Assert.Null(exception);

            if (System.IO.File.Exists(tempDb)) System.IO.File.Delete(tempDb);
        }

        [Fact]
        public void DeleteCommand_WithNullParameter_ShouldNotThrow()
        {
            // Arrange
            string tempDb = System.IO.Path.Combine(System.IO.Path.GetTempPath(), System.Guid.NewGuid().ToString() + ".json");
            var service = new GlossaryService(tempDb);
            var viewModel = new GlossaryViewModel(service);
            
            // Act & Assert
            var exception = Record.Exception(() => viewModel.DeleteCommand.Execute(null));
            
            Assert.Null(exception);
            
            if (System.IO.File.Exists(tempDb)) System.IO.File.Delete(tempDb);
        }

        [Fact]
        public void DeleteCommand_WithValidParameter_ShouldCallService()
        {
            // Arrange
            string tempDb = System.IO.Path.Combine(System.IO.Path.GetTempPath(), System.Guid.NewGuid().ToString() + ".json");
            var service = new GlossaryService(tempDb);
            var entry = new GlossaryEntry { SourceTerm = "Test", TargetTerm = "テスト", SourceLang = "en", TargetLang = "ja" };
            service.AddEntry(entry);
            
            var viewModel = new GlossaryViewModel(service);
            var entryInCollection = viewModel.Entries.First();

            // Act
            viewModel.DeleteCommand.Execute(entryInCollection);

            // Assert
            Assert.Empty(viewModel.Entries);
            Assert.Empty(service.GetAllEntries());

            if (System.IO.File.Exists(tempDb)) System.IO.File.Delete(tempDb);
        }
    }
}
