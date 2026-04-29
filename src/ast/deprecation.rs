// AST types for the `DEPRECATED[name_path, "msg"];` pragma.

use crate::ast::name::{FullName, NameSpace};
use crate::parse::sourcefile::Span;

/// A `DEPRECATED[...]` statement as parsed.
///
/// The path written by the user is interpreted relative to the surrounding
/// container (top level / `namespace { .. }` / `trait { .. }` body). At parse
/// time the relative path is concatenated with that container's namespace to
/// build the final target `FullName`, so `target_path` is what gets looked up
/// directly in `Program::global_values` / the trait-member registry during
/// elaboration. The exception is `DEPRECATED[::Foo::bar, ..]`: the absolute
/// marker is preserved on `target_path.namespace.is_absolute` so elaboration
/// can detect and reject it with a friendly error.
#[derive(Clone)]
pub struct DeprecationStatement {
    /// The fully-qualified target name, formed at parse time by concatenating
    /// the user-written relative path with `origin_namespace`. Looked up
    /// directly against `Program::global_values` / trait-member registry in
    /// elaboration. If the user wrote an absolute path (leading `::`), the
    /// value retains `namespace.is_absolute = true` so it can be rejected.
    pub target_path: FullName,
    /// The span of the trailing name token only (e.g. `bar` in
    /// `DEPRECATED[Foo::bar, ..]`), used for LSP rename / find-references.
    pub target_name_src: Option<Span>,
    /// The container path the relative path was written against. Used only
    /// for diagnostic messages ("not found under <container>"); resolution
    /// itself uses `target_path` directly.
    ///
    ///   - top level             -> root (empty)
    ///   - `namespace Foo { .. }` -> `Foo`
    ///   - `namespace Foo { trait a : MyTrait { .. } }` -> `Foo::MyTrait`
    ///
    /// A trait body contributes one extra component to the container path so
    /// we don't need a separate field for the enclosing trait.
    pub origin_namespace: NameSpace,
    /// The user-provided deprecation message.
    pub message: String,
    /// The span of the whole `DEPRECATED[...]` pragma.
    pub src: Option<Span>,
}

/// Per-symbol deprecation metadata, attached to the resolved `GlobalValue` or
/// `TraitMember` after elaboration.
#[derive(Clone)]
pub struct DeprecationInfo {
    /// The user-provided deprecation message.
    pub message: String,
}
