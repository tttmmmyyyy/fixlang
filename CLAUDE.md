# Claude Instructions

## Project Overview

This project implements the Fix programming language compiler and related tools as the `fix` command.

## Language and Code Style

- **Programming Language**: This project is written in Rust.
- **Fix Language Syntax**: Refer to `Document.md` for documentation and `src/tests/test_basic.rs` for Fix code examples.
- **Data Structures**:
  - Use `Set` and `Map` from `crate::misc` module instead of `std::collections::HashSet` and `std::collections::HashMap`.
  - Example: `use crate::misc::{Set, Map};` then use `Set::default()` or `Map::default()`.
- **Testability abstractions**: Do NOT complicate the code or introduce abstractions solely to make it unit-testable. Abstractions introduced solely for unit-testability tend to make the code harder to read.

## Testing Guidelines

- **When modifying Fix grammar or standard library**:
  - Add tests that compile and execute Fix code.
  - These tests verify that Fix language features work correctly.
  - **Always reference the thing under test from `main`**: When writing a test that checks whether some Fix code compiles, do NOT just declare/define the global value or trait member you want to verify. The test must actually use it from `main` — call the function, evaluate the value (using `eval` if direct calling is awkward), or otherwise reference it. Otherwise the symbol may be skipped by the compiler and a broken definition will not produce an error.
  
- **When modifying `fix` command behavior**:
  - Prefer integration tests.
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

## Measuring Performance

- **Judge neutrality from the binary first.** Build the same program before and after in the same
  directory and compare `objcopy -O binary --only-section=.text --only-section=.rodata`. Identical
  output is a neutral change and needs no measurement. Delete `.fixlang` before building. Where
  they differ, read `--emit-llvm`'s `_optimized.ll`.
- **A change that could not be shown neutral is measured before the pull request**, with
  `benchmark/speedtest` run as `--langarena` so that all 106 programs land in one row.
- **The primary metric is the instruction count (`instructions:u`).** Conclusions rest on it. It
  depends on neither the load on the machine nor where the code lands in `.text`.
- **The cycle count is the secondary metric**, reported beside the instruction count in the pull
  request, and read on a machine with nothing else running. Where a hot loop starts inside a
  64-byte line decides it, and what decides that is the sizes of everything the linker placed
  first: measured with the code held byte-identical, one benchmark spans 32.6% (#654). Read
  `idq_uops_not_delivered.core` beside it to tell the two apart — a cycle count that carries that
  counter with it is the front end being fed at another rate, which is what an address does, and
  one that leaves it where it was is the work.
- **Wall time says nothing the cycle count does not.** The two differ by a frequency that holds to
  within a percent where nothing else runs, so the address above reaches it just the same.
- **The report covers every case measured, the neutral ones included.** Listing only what moved
  makes a change look larger than it is; what says how much a gain or a regression weighs is how
  many of how many moved.

## Finishing a Change

- **When the implementation is complete**, run these skills in order before the work is handed over:
  1. `code-review` over the change, and act on the findings it reports.
  2. `bug-hunt` against the subsystem the change touched.
  3. `pr-message` to write the pull request body, then open the pull request.
  4. Put the pull request's number into the changelog entries the change adds, and push.
- **A change confined to the development tools takes steps 3 and 4 alone** — the benchmark
  harnesses under `benchmark/`, and anything else that no compiled program depends on. A defect
  there shows itself in the numbers the tool prints, to the one person reading them, and is
  corrected by running it again; the two review passes are for the code that ships to users.

## Changelog

- **When a round of modifications is complete**, add an entry describing the change to `CHANGELOG.md`.
  - Add it under the `## [Unreleased]` section at the top, in the appropriate category (`### Added` / `### Changed`) and subcategory (`#### Language` / `#### Tool` / `#### Std`), following the style of existing entries.
  - **Open the entry with the numbers behind it**: the issues the change closes and the pull request that carries it, in ascending order, then a colon — `- #297, #316: Two global values of one namespace whose names differ only in ...`. Write the numbers that exist: an entry with no issue names its pull request alone, and one whose change has neither carries no numbers. The pull request's number is added once the pull request is open (see "Finishing a Change").
  - **Performance improvements that do not change observable behavior do NOT need a changelog entry.** The changelog documents user-visible changes (new/changed/fixed behavior), not internal speedups.
