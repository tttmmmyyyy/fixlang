# Copilot Instructions

## Project Overview

This project implements the Fix programming language compiler and related tools as the `fix` command.

## Language and Code Style

- **Programming Language**: This project is written in Rust.
- **Fix Language Syntax**: Refer to `Document.md` for documentation and `src/tests/test_basic.rs` for Fix code examples.
- **Import Style**: 
  - Use explicit imports at the top of each file to reference identifiers without namespace qualification.
  - Do NOT use `module::*` wildcard imports.
  - Import all entities explicitly (e.g., `use module::{Type1, Type2, function1};`).
    - Note: Some existing code may not follow this style, but new code should adhere to it.

