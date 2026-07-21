// Memory-safety tests for the value a `match` binding carries.
// A match consumes its scrutinee at the head of the arm it takes, and an arm whose payload aliases
// the scrutinee — an unboxed union's variant slot, or a catch-all arm's whole scrutinee — carries
// that reference away into the match binding. The binding therefore names the same object as the
// scrutinee on those paths and a different one on the others, so its release drops a reference the
// scrutinee's own reference counting must not be optimized against. Each program below drops such a
// binding and then reads the scrutinee, so a reference dropped twice or released too early is a
// use-after-free — checked under valgrind.

#[cfg(test)]
mod match_result_alias_tests {
    use crate::{
        configuration::{Configuration, ValgrindTool},
        misc::{function_name, platform_valgrind_supported},
        tests::test_util::test_source,
    };

    const MATCH_RESULT_ALIAS_SOURCE: &str = r#"
module Main;

type UnboxOpt = unbox union { some : Array I64, none : () };
type BoxOpt = box union { some : Array I64, none : () };

main : IO () = (
    // An unboxed union: the arm payload is the scrutinee's variant slot, so the match binding
    // carries the scrutinee's reference away. The scrutinee is read after the binding is dropped.
    let u1 = UnboxOpt::some([1, 2, 3]);
    let v1 = match u1 { some(a) => a, none(_) => [] };
    assert_eq(|_|"unboxed, binding read", v1.@size, 3);;
    assert_eq(|_|"unboxed, scrutinee after", u1.as_some.@size, 3);;

    // `Std::Option` is that same shape.
    let u2 = Option::some([4, 5]);
    let v2 = match u2 { some(a) => a, none(_) => [] };
    assert_eq(|_|"option, binding read", v2.@size, 2);;
    assert_eq(|_|"option, scrutinee after", u2.as_some.@size, 2);;

    // The arm not taken produces a value of its own, so the binding is one object on one path and
    // another on the other.
    let u3 : UnboxOpt = UnboxOpt::none();
    let v3 = match u3 { some(a) => a, none(_) => [9, 9] };
    assert_eq(|_|"unboxed, arm not taken", v3.@size, 2);;
    assert_eq(|_|"unboxed, none scrutinee after", u3.is_some, false);;

    // A boxed union with a catch-all arm: the arm binds the whole scrutinee, so the binding carries
    // the scrutinee away just as an unboxed union's payload does.
    let u4 = BoxOpt::some([6, 7, 8, 9]);
    let v4 = match u4 { none(_) => BoxOpt::none(), rest => rest };
    assert_eq(|_|"catch-all, binding read", match v4 { some(a) => a.@size, none(_) => 0 }, 4);;
    assert_eq(|_|"catch-all, scrutinee after", match u4 { some(a) => a.@size, none(_) => 0 }, 4);;

    // The binding is read twice, so it is retained and released on top of the scrutinee's own
    // reference counting.
    let u5 = UnboxOpt::some([10, 20]);
    let v5 = match u5 { some(a) => a, none(_) => [] };
    assert_eq(|_|"binding read twice", v5.@(0) + v5.@(1), 30);;
    assert_eq(|_|"scrutinee after two reads", u5.as_some.@size, 2);;

    // The binding is consumed and then read again, so the retain that covers the consume stands
    // until the read releases it.
    let u6 = UnboxOpt::some([1, 2, 3]);
    let v6 = match u6 { some(a) => a, none(_) => [] };
    let s6 = v6.set(0, 10).@(0);
    assert_eq(|_|"binding consumed then read", s6 + v6.@size, 13);;

    pure()
);
"#;

    // A match binding that flows out of the function it is built in: the caller then holds a value
    // that is the argument's payload on one path and a fresh value on the other.
    const MATCH_RESULT_RETURNED_SOURCE: &str = r#"
module Main;

type UnboxOpt = unbox union { some : Array I64, none : () };

// Self-recursive so it is compiled as its own function rather than inlined into `main`.
payload_or_default : I64 -> UnboxOpt -> Array I64 = |n, u| (
    if n < 0 {
        payload_or_default(n + 1, UnboxOpt::none())
    } else {
        match u { some(a) => a, none(_) => [0] }
    }
);

// The argument is read after the returned value is dropped.
size_then_size : UnboxOpt -> I64 = |u| (
    let taken = payload_or_default(0, u);
    let n = taken.@size;
    n + u.as_some.@size
);

main : IO () = (
    assert_eq(|_|"returned payload", size_then_size(UnboxOpt::some([1, 2, 3])), 6);;
    assert_eq(|_|"returned default", payload_or_default(0, UnboxOpt::none()).@size, 1);;
    pure()
);
"#;

    #[test]
    pub fn test_match_result_alias_correctness() {
        test_source(MATCH_RESULT_ALIAS_SOURCE, Configuration::develop_mode());
    }

    #[test]
    pub fn test_match_result_alias_memory_safety() {
        if !platform_valgrind_supported() {
            eprintln!(
                "Skipping {}: Valgrind not available on this platform.",
                function_name!()
            );
            return;
        }
        let mut config = Configuration::develop_mode();
        config.set_valgrind(ValgrindTool::MemCheck);
        test_source(MATCH_RESULT_ALIAS_SOURCE, config);
    }

    #[test]
    pub fn test_match_result_returned_correctness() {
        test_source(MATCH_RESULT_RETURNED_SOURCE, Configuration::develop_mode());
    }

    #[test]
    pub fn test_match_result_returned_memory_safety() {
        if !platform_valgrind_supported() {
            eprintln!(
                "Skipping {}: Valgrind not available on this platform.",
                function_name!()
            );
            return;
        }
        let mut config = Configuration::develop_mode();
        config.set_valgrind(ValgrindTool::MemCheck);
        test_source(MATCH_RESULT_RETURNED_SOURCE, config);
    }
}
