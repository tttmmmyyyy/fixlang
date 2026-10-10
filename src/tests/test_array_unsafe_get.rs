// Tests for the `Array::unsafe_get_bounds_unchecked` builtin, which reads an element like `@` but
// omits the bounds check, and for its deprecated alias `Array::_unsafe_get_bounds_unchecked`.

#[cfg(test)]
mod array_unsafe_get_tests {
    use crate::{configuration::Configuration, tests::test_util::test_source};

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
}
