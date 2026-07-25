use crate::{
    configuration::{Configuration, FixOptimizationLevel},
    env_vars,
    tests::test_util::test_source,
};

// A `fix` self-call dispatches through a function pointer, which LLVM's tail-call elimination cannot
// fold into a loop when the return value uses the `sret` ABI (four or more scalar leaves). Rewriting
// `fix` into a direct self-recursive global makes the self-call direct, so a deeply tail-recursive
// `fix` returning such a value runs in constant stack. This drives that path with a return of
// `(I64, Array I64)` — four leaves once an array is three (pointer, size, capacity) — recursing a
// million deep, which overflows the stack unless the self-call is direct.
#[test]
fn test_deep_fix_sret_return_runs_in_constant_stack() {
    // `None` deliberately keeps even tail calls, so a `fix` tail-recursion overflows there by design;
    // the defunctionalization under test applies at `Basic` and above.
    if env_vars::get_max_opt_level() <= FixOptimizationLevel::None {
        return;
    }
    let source = r#"
    module Main;

    count : I64 -> (I64, Array I64);
    count = |n| (
        let go = fix(|go, i, arr| (
            if i >= n { (i, arr) };
            go(i + 1, arr)
        ));
        go(0, Array::empty(0))
    );

    main : IO ();
    main = (
        let (last, arr) = count(1000000);
        assert_eq(|_|"unexpected result", last + arr.@size, 1000000);;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_mode());
}
