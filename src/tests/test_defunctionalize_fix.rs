use crate::{
    configuration::{Configuration, FixOptimizationLevel},
    env_vars,
    tests::test_util::test_source,
};

// A `fix` self-call dispatches through a function pointer, which LLVM's tail-call elimination cannot
// fold into a loop when the return value uses the `sret` ABI (four or more scalar leaves). Rewriting
// `fix` into a direct self-recursive global makes the self-call direct, so a deeply tail-recursive
// `fix` returning such a value runs in constant stack. Each `*_runs_in_constant_stack` test drives
// that with a return of `(I64, Array I64)` — four leaves once an array is three (pointer, size,
// capacity) — recursing a million deep, which overflows the stack unless the self-call is direct.
//
// `None` deliberately keeps even tail calls, so the constant-stack tests skip it; the correctness
// test below runs at every level, since the defunctionalization must preserve results everywhere.

fn skip_at_none() -> bool {
    env_vars::get_max_opt_level() <= FixOptimizationLevel::None
}

// The `fix` argument written inline, the common idiom.
#[test]
fn test_deep_fix_sret_return_runs_in_constant_stack() {
    if skip_at_none() {
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

// The `fix` argument is a `let`-bound lambda applied to `fix` at two sites, so it cannot be inlined
// into either call; the pass resolves the binding and lifts each site.
#[test]
fn test_multi_use_let_bound_fix_runs_in_constant_stack() {
    if skip_at_none() {
        return;
    }
    let source = r#"
    module Main;

    run_twice : I64 -> I64;
    run_twice = |n| (
        let step : (I64 -> Array I64 -> (I64, Array I64)) -> I64 -> Array I64 -> (I64, Array I64)
            = |go, i, arr| (
                if i >= n { (i, arr) };
                go(i + 1, arr)
            );
        let (a, _) = (fix(step))(0, Array::empty(0));
        let (b, _) = (fix(step))(0, Array::empty(0));
        a + b
    );

    main : IO ();
    main = (
        assert_eq(|_|"unexpected result", run_twice(1000000), 2000000);;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_mode());
}

// The `fix` argument is a global function rather than an inline lambda.
#[test]
fn test_global_function_fix_runs_in_constant_stack() {
    if skip_at_none() {
        return;
    }
    let source = r#"
    module Main;

    gloop : (I64 -> Array I64 -> (I64, Array I64)) -> I64 -> Array I64 -> (I64, Array I64);
    gloop = |go, i, arr| (
        if i >= 1000000 { (i, arr) };
        go(i + 1, arr)
    );

    main : IO ();
    main = (
        let (last, arr) = (fix(gloop))(0, Array::empty(0));
        assert_eq(|_|"unexpected result", last + arr.@size, 1000000);;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_mode());
}

// Shape variety that the resolution and lifting must all compute correctly: a `fix` nested inside a
// `fix`, two distinct `fix` lambdas of the same type, a global fixed at two sites (deduplicated to one
// lifted function), a global that fixes itself and a pair that fix each other (deduplication must make
// compilation terminate), and a `fix` argument that is a `let`-bound value but not a bare lambda (left
// as a closure `fix`, so a heavy initializer would not be duplicated). It runs at every optimization
// level, none included.
#[test]
fn test_fix_resolution_variants_compute_correctly() {
    let source = r#"
    module Main;

    nested : I64;
    nested = (
        let outer = fix(|o, a| (
            if a >= 3 { 0 };
            let inner = fix(|i, b| ( if b >= 4 { 0 }; b + i(b + 1) ));
            inner(0) + o(a + 1)
        ));
        outer(0)
    );

    sum_to : I64 -> I64;
    sum_to = |n| (fix(|f, x| if x <= 0 { 0 } else { x + f(x - 1) }))(n);
    prod_to : I64 -> I64;
    prod_to = |n| (fix(|f, x| if x <= 1 { 1 } else { x * f(x - 1) }))(n);

    gloop : (I64 -> I64) -> I64 -> I64;
    gloop = |self, x| if x <= 0 { 0 } else { 1 + self(x - 1) };
    two_site : I64;
    two_site = (fix(gloop))(10) + (fix(gloop))(20);

    selffix : (I64 -> I64) -> I64 -> I64;
    selffix = |self, x| ( if x <= 0 { 0 }; x + fix(selffix)(x - 1) );
    self_res : I64;
    self_res = fix(selffix)(5);

    mutual_a : (I64 -> I64) -> I64 -> I64;
    mutual_a = |self, x| if x <= 0 { 0 } else { 1 + fix(mutual_b)(x - 1) };
    mutual_b : (I64 -> I64) -> I64 -> I64;
    mutual_b = |self, x| if x <= 0 { 0 } else { 1 + fix(mutual_a)(x - 1) };
    mutual_res : I64;
    mutual_res = fix(mutual_a)(6);

    not_a_bare_lambda : I64 -> I64;
    not_a_bare_lambda = |n| (
        let f : (I64 -> I64) -> I64 -> I64 = (
            let c = 42;
            |go, x| if x <= 0 { c } else { go(x - 1) }
        );
        (fix(f))(n)
    );

    main : IO ();
    main = (
        assert_eq(|_|"nested", nested, 18);;
        assert_eq(|_|"sum_to", sum_to(5), 15);;
        assert_eq(|_|"prod_to", prod_to(5), 120);;
        assert_eq(|_|"two_site", two_site, 30);;
        assert_eq(|_|"self_res", self_res, 15);;
        assert_eq(|_|"mutual_res", mutual_res, 6);;
        assert_eq(|_|"not_a_bare_lambda", not_a_bare_lambda(1000), 42);;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_mode());
}
