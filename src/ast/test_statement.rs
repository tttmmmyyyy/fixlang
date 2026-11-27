// AST types for the `TEST[name_path];` pragma.

use crate::ast::name::{FullName, NameSpace};
use crate::parse::sourcefile::Span;

/// A `TEST[...]` pragma, which declares a global value of type `IO ()` as a test `fix test` runs.
///
/// The path the pragma writes is relative to the namespace it is written in, as the path of a
/// `DEPRECATED[...]` pragma is (see `DeprecationStatement`).
#[derive(Clone)]
pub struct TestStatement {
    /// The global value the test runs: the path the pragma writes, joined under `origin_namespace`.
    /// A path written absolute keeps `namespace.is_absolute`, so that
    /// `Program::validate_test_statement_names` can reject it.
    pub value_name: FullName,
    /// The span of the last name of the path (`bar` in `TEST[Foo::bar];`), for the references the
    /// language server finds.
    pub value_name_src: Option<Span>,
    /// The namespace the pragma is written in.
    pub origin_namespace: NameSpace,
    /// Whether the pragma is marked `ignore`, which has `fix test` report the test as ignored. The
    /// value is type-checked as every global value is, and `fix test` neither generates its code
    /// nor runs it.
    pub ignore: bool,
    /// The span of the whole pragma.
    pub src: Option<Span>,
}
