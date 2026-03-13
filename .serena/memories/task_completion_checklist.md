# Task Completion Checklist

Before considering a task complete, ensure:
1. Solution builds without errors: `dotnet build`
2. All tests pass: `dotnet test` (Note: Ensure the application is not running if tests access shared files)
3. New symbols follow naming conventions (PascalCase for publics, _camelCase for privates)
4. Relevant documentation/README is updated if needed.
