# Copilot Instructions

## Project Overview

This project implements the Fix programming language compiler and related tools as the `fix` command.

## Language and Code Style

- **Programming Language**: This project is written in Rust.
- **Fix Language Syntax**: Refer to `Document.md` for documentation and `src/tests/test_basic.rs` for Fix code examples.
- **Comments**: Write all comments in English.
- **Import Style**: 
  - Use explicit imports at the top of each file to reference identifiers without namespace qualification.
  - Do NOT use `module::*` wildcard imports.
  - Import all entities explicitly (e.g., `use module::{Type1, Type2, function1};`).
    - Note: Some existing code may not follow this style, but new code should adhere to it.
- **Data Structures**:
  - Use `Set` and `Map` from `crate::misc` module instead of `std::collections::HashSet` and `std::collections::HashMap`.
  - Example: `use crate::misc::{Set, Map};` then use `Set::default()` or `Map::default()`.

## Testing Guidelines

- **When modifying Fix grammar or standard library**:
  - Add unit tests that compile and execute Fix code.
  - These tests verify that Fix language features work correctly.
  
- **When modifying `fix` command behavior**:
  - Do NOT write unit tests unless explicitly instructed.
  - Write integration tests instead.
  - Place sample Fix projects in the `tests` folder (e.g., `src/tests/test_dependencies/cases/`).
  - In test code, call `install_fix()` to install Fix to the system.
  - Test the actual behavior by running `fix` command via `Command::new("fix")`.
  - For tests that use Fix projects, always copy the project to a temporary directory using `setup_test_env()` pattern (see `test_dependencies.rs` for reference).
    - This ensures tests can run in parallel without conflicts.
    - Use `tempfile::TempDir` to create temporary directories.
    - Use `copy_dir_recursive()` from `test_util.rs` to copy project files.
    - The temporary directory is automatically cleaned up when the test completes.

## Reference Documentation

- **Fix Language Sample Code**: Refer to `src/fix/std.fix` for extensive examples of Fix language code.
- **Standard Library Documentation**: Refer to `std_doc/Std.md` for documentation of the Std standard library.
