// Tests for the `Array::unsafe_get_bounds_unchecked` builtin, which reads an element like `@` but
// omits the bounds check, and for its deprecated alias `Array::_unsafe_get_bounds_unchecked`.

#[cfg(test)]
mod array_unsafe_get_tests {
    use crate::{
        configuration::Configuration,
        tests::test_util::{deprecation_report, test_source},
    };

    /// Verifies that `unsafe_get_bounds_unchecked` and its deprecated alias read the element at the
    /// index, for unboxed elements at both ends and for a boxed element, and that reading a boxed
    /// element leaves the array intact.
    #[test]
    pub fn test_unsafe_get_bounds_unchecked() {
        let source = r#"
module Main;

main : IO ();
main = (
    // Unboxed and boxed elements, at both ends.
    let a = [10, 20, 30];
    assert_eq(|_|"unboxed first", a.unsafe_get_bounds_unchecked(0), 10);;
    assert_eq(|_|"unboxed last", a.unsafe_get_bounds_unchecked(2), 30);;
    let b = [[1], [2, 3]];
    assert_eq(|_|"boxed", b.unsafe_get_bounds_unchecked(1), [2, 3]);;
    // The array keeps the element it lent out.
    assert_eq(|_|"boxed array intact", b, [[1], [2, 3]]);;

    // The deprecated alias reads the same element.
    assert_eq(|_|"deprecated alias", a._unsafe_get_bounds_unchecked(1), 20);;
    pure()
);
"#;
        test_source(source, Configuration::develop_mode());
    }

    /// Naming `Array::_unsafe_get_bounds_unchecked` is reported as deprecated, with the message its
    /// `DEPRECATED` pragma carries pointing at the public name.
    #[test]
    pub fn test_unsafe_get_bounds_unchecked_private_name_is_deprecated() {
        let source = r#"
module Main;

main : IO ();
main = (
    assert_eq(|_|"deprecated alias", [10, 20, 30]._unsafe_get_bounds_unchecked(1), 20);;
    pure()
);
"#;
        let report = deprecation_report(source);
        assert!(
            report.contains("Use `Std::Array::unsafe_get_bounds_unchecked` instead."),
            "naming `_unsafe_get_bounds_unchecked` should be reported with the message its pragma carries:\n{}",
            report,
        );
    }
}
