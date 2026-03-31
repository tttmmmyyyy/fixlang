---
name: shorten-qualifiers
description: "Shorten fully-qualified identifiers in Rust code. Use when: reviewing diffs for verbose paths like crate::module::Type, replacing qualified names with imports, cleaning up namespace-heavy code. Analyzes unstaged/uncommitted git diff and adds use statements to eliminate unnecessary qualification."
argument-hint: "Optionally specify file paths to limit scope, or leave empty to process entire diff"
---

# Shorten Fully-Qualified Identifiers

Replace verbose qualified paths (e.g., `crate::foo::bar::Baz`) with short names by adding `use` imports.

## When to Use

- After writing or generating new code that uses fully-qualified paths
- During code review to clean up verbose identifiers
- When the user says "shorten", "import", "qualify", or "use短く"

## Procedure

1. **Collect changed files**: Run `git diff --name-only` (unstaged) and `git diff --cached --name-only` (staged) to find affected files. If the user specifies files, restrict to those files.

2. **Identify qualified paths**: For each affected file, search the **entire file** (not just diff lines) for Rust paths with `::` that could be shortened. The diff is used only to determine *which files* to process, not to limit the search scope within those files. This ensures that pre-existing qualified paths in the same file are also cleaned up. Look for patterns like:
   - `crate::module::Ident`
   - `crate::module::{A, B}`
   - `module::submodule::Ident`
   - Paths used as types, function calls, trait bounds, or in expressions.

3. **Read each file's existing imports**: For each affected file, read the `use` block at the top to know what is already imported.

4. **Plan replacements**: For each qualified path found:
   - Determine the short name (last segment, e.g., `Baz` from `crate::foo::bar::Baz`).
   - Check if the short name conflicts with another import or a different qualified path in the same file.
     - **No conflict**: Add a `use` statement and replace all occurrences with the short name.
     - **Conflict**: Keep the minimal qualification needed to disambiguate (e.g., `bar::Baz` instead of full `crate::foo::bar::Baz`).
   - If an existing `use` already imports from the same module, extend it (e.g., `use crate::foo::{A};` → `use crate::foo::{A, B};`).

5. **Apply edits**:
   - Add new `use` statements in the import block, following the file's existing style.
   - Replace qualified paths in the code with short names.
   - Follow the project convention from copilot-instructions.md: use explicit imports, no wildcard `*` imports.

6. **Verify**: Build the project (`cargo build`) to confirm no compilation errors.

## Collision Detection

A name collision exists when two different fully-qualified paths resolve to the same short name. For example:
- `crate::ast::types::TypeNode` and `crate::parse::types::TypeNode` both shorten to `TypeNode`.

In this case, keep one as a `use` import and qualify the other minimally, or qualify both if the file uses them equally.

## Edge Cases

- **Re-exports**: If `crate::module` re-exports a type, prefer the shorter re-export path.
- **Already imported**: If the identifier is already imported, just replace the qualified usage; don't add a duplicate import.
- **Inside macro invocations**: Be cautious with paths inside macros; they may require full qualification.
- **Non-`crate` paths**: Also handle `std::`, `serde::`, etc. external crate paths if they appear qualified in code.
