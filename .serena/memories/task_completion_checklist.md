# Task Completion Checklist

Before considering a task complete, ensure:
1. Rust backend compiles without errors: `cd src-tauri && cargo build`
2. All Rust tests pass: `cd src-tauri && cargo test`
3. App launches successfully: `npm run tauri dev` (verify window appears and basic operations work)
4. New Rust symbols follow naming conventions (snake_case for functions/variables, PascalCase for types)
5. Relevant documentation/README is updated if needed.
6. No debug artifacts (log files, test outputs) remain in the repository.