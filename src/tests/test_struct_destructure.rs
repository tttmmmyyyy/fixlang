// Memory-safety tests for struct-pattern destructuring of a boxed struct with boxed fields, and for a
// parameter whose only use is such a destructure.
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

    /// Destructures of a boxed struct whose fields are boxed, in each shape that decides how many
    /// times a value is retained and released: the container's last use, a use of the container
    /// after the destructure, a field the continuation drops, and a pattern reaching into an inner
    /// boxed struct.
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

    /// A function whose only use of a boxed parameter is to destructure it, called twice on one
    /// value the caller keeps using.
    ///
    /// A destructure consumes its container, so ownership inference has to give the parameter to
    /// the callee: a callee that borrowed it would release a container it does not own, and the
    /// caller's value would die while the caller still holds it. The recursion keeps the callee out
    /// of the caller, so the call goes through the inferred parameter ownership.
    const DESTRUCTURED_PARAMETER_SOURCE: &str = r#"
module Main;

type BoxPair = box struct { a : Array I64, b : Array I64 };

sum_heads : I64 -> BoxPair -> I64 = |n, p| (
    if n <= 0 {
        let BoxPair { a: x, b: y } = p;
        x.@(0) + y.@(0)
    } else {
        sum_heads(n - 1, p)
    }
);

main : IO () = (
    let p = BoxPair { a: [1, 2], b: [10, 20] };
    assert_eq(|_|"first call", sum_heads(2, p), 11);;
    assert_eq(|_|"second call", sum_heads(2, p), 11);;
    assert_eq(|_|"container intact", p.@a.@(1) + p.@b.@(1), 22);;
    pure()
);
"#;

    /// Destructuring a boxed struct of boxed fields binds each field to its value: where the
    /// destructure is the container's last use, where the container is used after it, where one of
    /// the fields is dropped, and where the pattern reaches into an inner boxed struct.
    #[test]
    pub fn test_boxed_struct_destructure_correctness() {
        test_source(BOXED_DESTRUCTURE_SOURCE, Configuration::develop_mode());
    }

    /// The destructures of `BOXED_DESTRUCTURE_SOURCE` leave every value released exactly once,
    /// checked under valgrind. A dropped field and a container used after the destructure are
    /// where a retain or a release too few shows itself.
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

    /// A function whose only use of a boxed parameter is to destructure it answers with the field
    /// values of the argument it was given, and the caller's value serves the calls that follow.
    #[test]
    pub fn test_destructured_parameter_correctness() {
        test_source(DESTRUCTURED_PARAMETER_SOURCE, Configuration::develop_mode());
    }

    /// An argument passed to a function that destructures it stays valid for the caller's later
    /// uses, checked under valgrind: a callee that released a container it does not own would kill
    /// the caller's value while the caller still holds it.
    #[test]
    pub fn test_destructured_parameter_memory_safety() {
        if !platform_valgrind_supported() {
            eprintln!(
                "Skipping {}: Valgrind not available on this platform.",
                function_name!()
            );
            return;
        }
        let mut config = Configuration::develop_mode();
        config.set_valgrind(ValgrindTool::MemCheck);
        test_source(DESTRUCTURED_PARAMETER_SOURCE, config);
    }

    /// Struct patterns that name a strict subset of the fields, and patterns that name every field
    /// in an order other than the declared one, over a boxed and an unboxed container whose
    /// unnamed fields are boxed.
    ///
    /// A partial pattern is what splits the two halves of `get_struct_fields`: an unboxed
    /// container hands out the named fields and releases the ones left behind, while a boxed one
    /// retains the named fields and releases itself, letting its own drop reach the rest. A
    /// reordered pattern is what makes the extraction read the field index recorded per field
    /// rather than the position the field is written at.
    const PARTIAL_DESTRUCTURE_SOURCE: &str = r#"
module Main;

type UnboxTriple = unbox struct { a : Array I64, b : Array I64, c : I64 };
type BoxTriple = box struct { a : Array I64, b : Array I64, c : I64 };

main : IO () = (
    // unboxed container, one boxed field named: the two left behind are released here
    let u1 = UnboxTriple { a: [1, 2], b: [3, 4], c: 5 };
    let UnboxTriple { a: ua } = u1;
    assert_eq(|_|"unbox partial", ua.@(1), 2);;

    // unboxed container used after a partial destructure
    let u2 = UnboxTriple { a: [6], b: [7, 8], c: 9 };
    let UnboxTriple { b: ub } = u2;
    assert_eq(|_|"unbox partial kept", ub.@(1) + u2.@a.@(0) + u2.@c, 23);;

    // boxed container, one boxed field named: the container's own drop reaches the rest
    let b1 = BoxTriple { a: [10, 11], b: [12], c: 13 };
    let BoxTriple { b: bb } = b1;
    assert_eq(|_|"box partial", bb.@(0), 12);;

    // boxed container used after a partial destructure
    let b2 = BoxTriple { a: [14], b: [15, 16], c: 17 };
    let BoxTriple { a: ba } = b2;
    assert_eq(|_|"box partial kept", ba.@(0) + b2.@b.@(1) + b2.@c, 47);;

    // every field named, written in an order other than the declared one
    let u3 = UnboxTriple { a: [18], b: [19, 20], c: 21 };
    let UnboxTriple { c: uc3, b: ub3, a: ua3 } = u3;
    assert_eq(|_|"unbox reordered", ua3.@(0) + ub3.@(1) + uc3, 59);;

    let b3 = BoxTriple { a: [22], b: [23, 24], c: 25 };
    let BoxTriple { c: bc3, b: bb3, a: ba3 } = b3;
    assert_eq(|_|"box reordered", ba3.@(0) + bb3.@(1) + bc3, 71);;

    pure()
);
"#;

    /// A pattern naming a strict subset of the fields binds those fields to their values, and a
    /// pattern naming the fields out of declaration order binds each name to the field it names.
    #[test]
    pub fn test_partial_and_reordered_destructure_correctness() {
        test_source(PARTIAL_DESTRUCTURE_SOURCE, Configuration::develop_mode());
    }

    /// The destructures of `PARTIAL_DESTRUCTURE_SOURCE` leave every value released exactly once,
    /// checked under valgrind. A field a pattern leaves unnamed is where a release too few (a leak)
    /// or one too many (a double free) shows itself.
    #[test]
    pub fn test_partial_and_reordered_destructure_memory_safety() {
        if !platform_valgrind_supported() {
            eprintln!(
                "Skipping {}: Valgrind not available on this platform.",
                function_name!()
            );
            return;
        }
        let mut config = Configuration::develop_mode();
        config.set_valgrind(ValgrindTool::MemCheck);
        test_source(PARTIAL_DESTRUCTURE_SOURCE, config);
    }
}

// A struct pattern names the fields of one struct type, so a name the type does not declare and a
// name given twice are both rejected with a source-level diagnostic.
#[cfg(test)]
mod struct_pattern_validation_tests {
    use crate::{
        configuration::Configuration,
        tests::test_util::{run_source_assert_failed, test_source_fail},
    };

    /// A pattern that both repeats a field and names one the struct does not declare is reported
    /// for both, so one compilation shows every way the field list is wrong, as it does for a
    /// struct literal.
    #[test]
    pub fn test_struct_pattern_duplicate_and_unknown_fields_both_reported() {
        let source = r#"
module Main;

type S = struct { a : I64, b : I64 };

main : IO ();
main = (
    let S { a : x, a : y, zz : z } = S { a : 1, b : 2 };
    println((x + y).to_string)
);
"#;
        let errmsg = run_source_assert_failed(source, Configuration::develop_mode());
        assert!(
            errmsg.contains("Duplicate field `a` of struct `Main::S`."),
            "the repeated field is reported, but the message is:\n{}",
            errmsg
        );
        assert!(
            errmsg.contains("Unknown field `zz` for struct `Main::S`."),
            "the undeclared field is reported, but the message is:\n{}",
            errmsg
        );
    }

    /// A field named twice in one struct pattern is reported, with the field and the struct named.
    #[test]
    pub fn test_struct_pattern_duplicate_field_rejected() {
        let source = r#"
module Main;

type S = struct { a : I64, b : I64 };

main : IO ();
main = (
    let S { a : x, a : y } = S { a : 1, b : 2 };
    println((x + y).to_string)
);
"#;
        test_source_fail(
            source,
            Configuration::develop_mode(),
            "Duplicate field `a` of struct `Main::S`.",
        );
    }

    /// A field name the struct does not declare is reported as an unknown field of that struct.
    #[test]
    pub fn test_struct_pattern_unknown_field_rejected() {
        let source = r#"
module Main;

type S = struct { a : I64 };

main : IO ();
main = (
    let S { zz : x } = S { a : 1 };
    println(x.to_string)
);
"#;
        test_source_fail(
            source,
            Configuration::develop_mode(),
            "Unknown field `zz` for struct `Main::S`.",
        );
    }
}
