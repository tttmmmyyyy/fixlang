// Memory-safety tests for struct-pattern destructuring of a boxed struct with boxed fields.
// Destructuring extracts the fields with `get_struct_fields`, whose boxed-container path retains
// each extracted field and releases the container. With boxed fields, a field the continuation
// drops, and a container still used after the destructure, must leave every value released exactly
// once — checked under valgrind. `test_basic`'s boxed-struct pattern test uses unboxed `I64` fields,
// which do not exercise the field retains; these tests use boxed (`Array`) fields.

#[cfg(test)]
mod struct_destructure_tests {
    use crate::{
        configuration::{Configuration, ValgrindTool},
        misc::{function_name, platform_valgrind_supported},
        tests::test_util::test_source,
    };

    const BOXED_DESTRUCTURE_SOURCE: &str = r#"
module Main;

type BoxPair = box struct { a : Array I64, b : Array I64 };
type BoxNest = box struct { p : BoxPair, tag : Array I64 };

main : IO () = (
    // container last-use, both boxed fields used (move-out)
    let p1 = BoxPair { a: [1, 2], b: [3, 4] };
    let BoxPair { a: a1, b: b1 } = p1;
    assert_eq(|_|"both", a1.@(0) + b1.@(1), 5);;

    // container last-use, one boxed field dropped (must be released, not leaked or double-freed)
    let p2 = BoxPair { a: [5, 6], b: [7, 8] };
    let BoxPair { a: a2, b: b2 } = p2;
    assert_eq(|_|"one dropped", a2.@(1), 6);;

    // container still used after the destructure, one boxed field dropped
    let p3 = BoxPair { a: [9], b: [10, 11] };
    let BoxPair { a: a3, b: b3 } = p3;
    assert_eq(|_|"kept field", a3.@(0), 9);;
    assert_eq(|_|"kept container", p3.@a.@(0) + p3.@b.@(1), 20);;

    // container destructured twice (the first use retains it, the second is its last use)
    let p4 = BoxPair { a: [100], b: [200] };
    let BoxPair { a: a4a, b: b4a } = p4;
    let BoxPair { a: a4b, b: b4b } = p4;
    assert_eq(|_|"shared", a4a.@(0) + b4a.@(0) + a4b.@(0) + b4b.@(0), 600);;

    // nested boxed struct: the inner boxed struct is destructured recursively
    let n = BoxNest { p: BoxPair { a: [1], b: [2] }, tag: [9] };
    let BoxNest { p: BoxPair { a: na, b: nb }, tag: nt } = n;
    assert_eq(|_|"nested", na.@(0) + nb.@(0) + nt.@(0), 12);;

    pure()
);
"#;

    #[test]
    pub fn test_boxed_struct_destructure_correctness() {
        test_source(BOXED_DESTRUCTURE_SOURCE, Configuration::develop_mode());
    }

    #[test]
    pub fn test_boxed_struct_destructure_memory_safety() {
        if !platform_valgrind_supported() {
            eprintln!(
                "Skipping {}: Valgrind not available on this platform.",
                function_name!()
            );
            return;
        }
        let mut config = Configuration::develop_mode();
        config.set_valgrind(ValgrindTool::MemCheck);
        test_source(BOXED_DESTRUCTURE_SOURCE, config);
    }
}
