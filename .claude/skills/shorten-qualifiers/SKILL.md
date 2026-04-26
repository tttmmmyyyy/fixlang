---
name: shorten-qualifiers
description: "Clean up Rust import style: shorten fully-qualified `crate::module::Type` paths by adding explicit `use` imports, replace any wildcard `use foo::*;` with explicit member lists, and collapse the file's `use` block into one contiguous group with no blank-line separators. Use when: reviewing import style, cleaning up namespace-heavy code, before committing new code."
argument-hint: "Base git ref to diff against; defaults to HEAD"
---

# Clean Up Rust Import Style

Three related cleanups:

1. **Shorten qualified paths** — replace `crate::foo::bar::Baz` with `Baz` plus a `use` import.
2. **Eliminate wildcard imports** — replace `use foo::*;` with an explicit list of the names actually used.
3. **Collapse the use block** — remove blank lines between `use` statements at the top of the file. The project convention is one contiguous block; sectioning (std vs external vs crate, as `rust-analyzer` likes to do) is not meaningful here.

The project convention is *explicit imports, no wildcards, no section breaks*. All three cleanups serve that convention.

## When to Use

- After writing or generating new code that uses fully-qualified paths or wildcard imports.
- During code review to clean up import style.
- When the user says "shorten", "import", "qualify", or "use短く".

## Procedure

1. **Collect changed files**: Run `git diff --name-only <base>` (where `<base>` = the argument, defaulting to `HEAD`) to find affected files.

2. **Identify cleanup targets**: For each affected file, search the **entire file** (not just diff lines). The diff is used only to determine *which files* to process — pre-existing violations in the same file are also fixed. Look for:
   - **Wildcard imports**: `use module::*;` (and grouped variants like `use module::{*}`).
   - **Qualified paths**: `crate::module::Ident`, `crate::module::{A, B}`, `module::submodule::Ident`. Paths used as types, function calls, trait bounds, or in expressions.
   - **Blank lines inside the top-of-file `use` block**: any empty line between two `use` statements at the start of the file. These are typically `rust-analyzer`-inserted section breaks (std / external crates / `crate` / `super`) that the project does not want.

3. **Read each file's existing imports**: For each affected file, read the `use` block at the top to know what is already imported.

4. **Plan replacements**:
   - **For each wildcard import** `use foo::*;`: list every identifier from `foo` actually referenced in the file, and rewrite the import as `use foo::{A, B, C};` (or merge into an existing line if one already imports from `foo`).
   - **For each qualified path**:
     - Determine the short name (last segment, e.g., `Baz` from `crate::foo::bar::Baz`).
     - Check if the short name conflicts with another import or a different qualified path in the same file.
       - **No conflict**: Add a `use` statement and replace all occurrences with the short name.
       - **Conflict**: Keep the minimal qualification needed to disambiguate (e.g., `bar::Baz` instead of full `crate::foo::bar::Baz`).
     - If an existing `use` already imports from the same module, extend it (e.g., `use crate::foo::{A};` → `use crate::foo::{A, B};`).
   - **For each blank line inside the top-of-file `use` block**: delete it. The block should run from the first `use` line to the last with no empty lines between. Do not reorder the imports — only remove the blanks. The single blank line that separates the entire `use` block from the code below it stays.

5. **Apply edits**: Add/update `use` statements in the import block, following the file's existing style. Replace qualified paths with short names.

6. **Verify**: Build the project (`cargo check`) to confirm no compilation errors. Wildcard removal is the most error-prone step — if some identifier was implicitly pulled in via the wildcard, the build will fail and reveal it; add it to the explicit list.

## Collision Detection

A name collision exists when two different fully-qualified paths resolve to the same short name. For example:
- `crate::ast::types::TypeNode` and `crate::parse::types::TypeNode` both shorten to `TypeNode`.

In this case, keep one as a `use` import and qualify the other minimally, or qualify both if the file uses them equally.

## Edge Cases

- **Re-exports**: If `crate::module` re-exports a type, prefer the shorter re-export path.
- **Already imported**: If the identifier is already imported, just replace the qualified usage; don't add a duplicate import.
- **Inside macro invocations**: Be cautious with paths inside macros; they may require full qualification.
- **Non-`crate` paths**: Also handle `std::`, `serde::`, etc. external crate paths if they appear qualified in code.
