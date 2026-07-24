// Memory-safety tests for destructuring an unboxed struct or tuple whose fields are boxed.
// `get_struct_fields`'s unboxed path moves each named field out of the container without a retain and
// releases every field the pattern omits; a field the pattern binds but the continuation never uses is
// released once as a dead binding. With boxed (`Array`) fields, a dropped field, and a container still
// read after the destructure, every value must be released exactly once — checked under valgrind.
// `test_struct_destructure` destructures a `box struct` (fields retained, container released), so the
// unboxed move-out and the dropped-field releases are exercised only here.

#[cfg(test)]
mod unbox_destructure_tests {
    use crate::{
        configuration::{Configuration, ValgrindTool},
        misc::{function_name, platform_valgrind_supported},
        tests::test_util::test_source,
    };

    const UNBOX_DESTRUCTURE_SOURCE: &str = r#"
module Main;

type UnboxPair = struct { a : Array I64, b : Array I64 };

main : IO () = (
    // tuple last-use, 2nd boxed field bound to a name the continuation never uses, so it is released
    // once as a dead binding (move-out then dead-binding release)
    let t = ([1, 2], [3, 4]);
    let (t0, dropped_b) = t;
    assert_eq(|_|"tuple drop 2nd", t0.@(0) + t0.@(1), 3);;

    // unbox struct last-use, partial pattern names only `a`, omitting boxed field `b` (drop-omitted)
    let p1 = UnboxPair { a: [5, 6], b: [7, 8] };
    let UnboxPair { a: a1 } = p1;
    assert_eq(|_|"struct drop b", a1.@(1), 6);;

    // unbox struct last-use, both fields named, `b` never used so it is released as a dead binding
    let p2 = UnboxPair { a: [9], b: [10, 11] };
    let UnboxPair { a: a2, b: dropped_b2 } = p2;
    assert_eq(|_|"struct unused binding", a2.@(0), 9);;

    // unbox struct still read after the destructure (shared), one boxed field dropped
    let p3 = UnboxPair { a: [12], b: [13, 14] };
    let UnboxPair { a: a3 } = p3;
    assert_eq(|_|"kept field", a3.@(0), 12);;
    assert_eq(|_|"kept container", p3.@a.@(0) + p3.@b.@(1), 26);;

    pure()
);
"#;

    #[test]
    pub fn test_unbox_destructure_correctness() {
        test_source(UNBOX_DESTRUCTURE_SOURCE, Configuration::develop_mode());
    }

    #[test]
    pub fn test_unbox_destructure_memory_safety() {
        if !platform_valgrind_supported() {
            eprintln!(
                "Skipping {}: Valgrind not available on this platform.",
                function_name!()
            );
            return;
        }
        let mut config = Configuration::develop_mode();
        config.set_valgrind(ValgrindTool::MemCheck);
        test_source(UNBOX_DESTRUCTURE_SOURCE, config);
    }
}
