# Claude Instructions

## Project Overview

This project implements the Fix programming language compiler and related tools as the `fix` command.

## Language and Code Style

- **Programming Language**: This project is written in Rust.
- **Fix Language Syntax**: Refer to `Document.md` for documentation and `src/tests/test_basic.rs` for Fix code examples.
- **Data Structures**:
  - Use `Set` and `Map` from `crate::misc` module instead of `std::collections::HashSet` and `std::collections::HashMap`.
  - Example: `use crate::misc::{Set, Map};` then use `Set::default()` or `Map::default()`.

## Testing Guidelines

- **When modifying Fix grammar or standard library**:
  - Add unit tests that compile and execute Fix code.
  - These tests verify that Fix language features work correctly.
  
- **When modifying `fix` command behavior**:
  - Do NOT write unit tests unless explicitly instructed.
    - Reason: abstractions introduced solely to make code unit-testable tend to make the code harder to read.
  - Write integration tests instead.
  - Place sample Fix projects in the `tests` folder (e.g., `src/tests/test_dependencies/cases/`).
  - In test code, call `install_fix()` to install Fix to the system.
  - Test the actual behavior by running `fix` command via `Command::new("fix")`.
  - For tests that use Fix projects, always copy the project to a temporary directory using `setup_test_env()` pattern (see `test_dependencies.rs` for reference).
    - This ensures tests can run in parallel without conflicts.
    - Use `tempfile::TempDir` to create temporary directories.
    - Use `copy_dir_recursive()` from `test_util.rs` to copy project files.
    - The temporary directory is automatically cleaned up when the test completes.
  - **Debugging integration tests**: Since integration tests run the `fix` command as a separate process, its stdout/stderr output is hard to capture directly. In such cases, use `WRITE_LOG` from `src/log_file.rs` to write debug output to a log file from within the `fix` process.

- **Running many tests at once**: When running a large number of tests with `cargo test` (e.g. the full suite or many integration tests), use `--release` (i.e. `cargo test --release`). Debug builds of the `fix` compiler are slow to run, so release mode significantly reduces total test time.

- **Failing tests**: Do NOT add `#[ignore]` to tests to bypass failures. Leaving failing tests in place and committing them is acceptable; hiding them with `#[ignore]` is not.

- **Dead-code warnings**: Do NOT add `#[allow(dead_code)]` to silence the "never used" warning on items that will eventually be used in production code (e.g. a constant or function added in one step of a multi-step rollout that will be consumed in a later step). The warning is the reminder that the follow-up work is still pending; suppressing it loses that signal. Leave the warning in place and let the next step resolve it.

## Reference Documentation

- **Fix Language Sample Code**: Refer to `src/fix/std.fix` for extensive examples of Fix language code.
- **Standard Library Documentation**: Refer to `std_doc/Std.md` for documentation of the Std standard library.

## Fix Language Specifics

- **Iterator `fold` function**: The closure passed to `fold` has the signature `(Item, Acc) -> Acc`, where the first argument is the current item and the second is the accumulator. This is the reverse of Haskell's `foldl`.
  - Correct: `iterator.fold(initial, |item, acc| acc + item)`
  - Incorrect: `iterator.fold(initial, |acc, item| acc + item)` (This will cause a type error)
