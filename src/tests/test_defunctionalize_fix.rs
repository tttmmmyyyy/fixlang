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

fn should_skip_at_none() -> bool {
    env_vars::get_max_opt_level() <= FixOptimizationLevel::None
}

// The `fix` argument written inline, the common idiom.
#[test]
fn test_deep_fix_sret_return_runs_in_constant_stack() {
    if should_skip_at_none() {
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
    if should_skip_at_none() {
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

// The `fix` argument is a global function.
#[test]
fn test_global_function_fix_runs_in_constant_stack() {
    if should_skip_at_none() {
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
// compilation terminate), a `fix` argument that is a `let`-bound value but not a bare lambda, and a
// `fix` applied to a function parameter — both left as a closure `fix`, the first so a heavy
// initializer is not duplicated, the second because the argument is only known at run time — and a
// recursion functional that passes `self` by value, exercising the partial-application closure the
// substitution leaves for a non-tail use. It runs at every optimization level, none included.
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

    apply_fix : ((I64 -> I64) -> I64 -> I64) -> I64 -> I64;
    apply_fix = |g, n| (fix(g))(n);
    fix_of_parameter : I64;
    fix_of_parameter = apply_fix(|self, x| if x <= 0 { 0 } else { x + self(x - 1) }, 5);

    applyit : (I64 -> I64) -> I64 -> I64;
    applyit = |h, x| h(x);
    self_escapes : I64;
    self_escapes = (fix(|self, x| if x <= 0 { 0 } else { applyit(self, x - 1) + 1 }))(5);

    main : IO ();
    main = (
        assert_eq(|_|"nested", nested, 18);;
        assert_eq(|_|"sum_to", sum_to(5), 15);;
        assert_eq(|_|"prod_to", prod_to(5), 120);;
        assert_eq(|_|"two_site", two_site, 30);;
        assert_eq(|_|"self_res", self_res, 15);;
        assert_eq(|_|"mutual_res", mutual_res, 6);;
        assert_eq(|_|"not_a_bare_lambda", not_a_bare_lambda(1000), 42);;
        assert_eq(|_|"fix_of_parameter", fix_of_parameter, 15);;
        assert_eq(|_|"self_escapes", self_escapes, 5);;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_mode());
}

// An inner `fix` that calls the outer `fix`'s self-reference. Lifting the outer `fix` names its
// capture parameter `#fixcap..`; the inner `fix` closes over that parameter, so the inner capture
// struct carries a field of the same name. The inner lifted function's own capture parameter must
// stay distinct from that field, or destructuring the capture shadows the parameter and the inner
// self-call forwards the outer capture struct in its place.
#[test]
fn test_nested_fix_inner_calls_outer_self() {
    let source = r#"
    module Main;

    main : IO ();
    main = (
        let f = fix(|outer, m|
            if m == 0 { 1 } else {
                let inner = fix(|inn, k| if k == 0 { outer(m - 1) } else { inn(k - 1) + 1 });
                inner(3)
            }
        );
        assert_eq(|_|"nested fix inner-calls-outer", f(2), 7);;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_mode());
}

// A captured boxed value crosses the direct self-call: the capture struct owns a reference, so the
// rewrite must keep retain/release balanced or the value leaks or is freed early. Under
// `develop_mode` the program runs under valgrind, which surfaces such an imbalance. Each function
// captures a boxed value read across the recursion and still live after the `fix` returns: an
// `Array` read every iteration, `self` escaping as a closure over the capture, one `let`-bound
// lambda fixed at two sites (each rebuilding its own capture of the shared array), and a nested
// `Array` of `Array`. It runs at every optimization level, none included.
#[test]
fn test_fix_boxed_capture_is_memory_safe() {
    let source = r#"
    module Main;

    sum_table : I64 -> Array I64 -> I64;
    sum_table = |depth, tbl| (
        let go = fix(|go, i, acc| (
            if i >= depth { acc };
            go(i + 1, acc + tbl.@(i % tbl.@size))
        ));
        go(0, 0)
    );

    applyit : (I64 -> I64) -> I64 -> I64;
    applyit = |h, x| h(x);
    escape_boxed : Array I64 -> I64;
    escape_boxed = |tbl| (
        (fix(|self, x| if x <= 0 { tbl.@(0) } else { applyit(self, x - 1) + tbl.@(x % tbl.@size) }))(4)
    );

    two_site_boxed : Array I64 -> I64;
    two_site_boxed = |tbl| (
        let step : (I64 -> I64) -> I64 -> I64
            = |go, i| ( if i >= 5 { 0 }; tbl.@(i % tbl.@size) + go(i + 1) );
        (fix(step))(0) + (fix(step))(0)
    );

    nested_boxed : Array (Array I64) -> I64;
    nested_boxed = |grid| (
        let go = fix(|go, i, acc| (
            if i >= 6 { acc + grid.@(0).@(0) };
            let row = grid.@(i % grid.@size);
            go(i + 1, acc + row.@(i % row.@size))
        ));
        go(0, 0)
    );

    main : IO ();
    main = (
        let tbl = Array::from_map(4, |i| i + 1);
        assert_eq(|_|"sum_table", sum_table(8, tbl) + tbl.@(0), 21);;
        assert_eq(|_|"escape_boxed", escape_boxed(tbl), 11);;
        assert_eq(|_|"two_site_boxed", two_site_boxed(tbl), 22);;
        let grid = Array::from_map(3, |i| Array::from_map(3, |j| i * 3 + j));
        assert_eq(|_|"nested_boxed", nested_boxed(grid) + grid.@(2).@(2), 32);;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_mode());
}

// The `fix` argument captures a variable introduced by a non-trivial binder: a `match`-arm pattern
// variable, a tuple destructure, and a struct destructure. Each captured name must appear in the
// lifted capture struct with the type it has in scope, so lifting must compute the same value as the
// closure form. A boxed array captured from a `match` arm and read across the recursion also checks
// reference-count balance under valgrind. It runs at every optimization level, none included.
#[test]
fn test_fix_captures_from_nontrivial_binders_compute_correctly() {
    let source = r#"
    module Main;

    type P = struct { fst : I64, snd : I64 };

    match_arm_capture : Option I64 -> I64;
    match_arm_capture = |opt| (
        match opt {
            some(v) => (
                let go = fix(|go, i| if i >= 5 { 0 } else { v + go(i + 1) });
                go(0)
            ),
            none() => -1
        }
    );

    tuple_capture : (I64, I64) -> I64;
    tuple_capture = |pair| (
        let (a, b) = pair;
        let go = fix(|go, i| if i >= 4 { 0 } else { a * b + go(i + 1) });
        go(0)
    );

    struct_capture : P -> I64;
    struct_capture = |p| (
        let P { fst : x, snd : y } = p;
        let go = fix(|go, i| if i >= 3 { 0 } else { x - y + go(i + 1) });
        go(0)
    );

    match_boxed_capture : Option (Array I64) -> I64;
    match_boxed_capture = |opt| (
        match opt {
            some(arr) => (
                let go = fix(|go, i| if i >= arr.@size { 0 } else { arr.@(i) + go(i + 1) });
                go(0) + arr.@(0)
            ),
            none() => -1
        }
    );

    main : IO ();
    main = (
        assert_eq(|_|"match_arm_capture", match_arm_capture(Option::some(7)), 35);;
        assert_eq(|_|"tuple_capture", tuple_capture((3, 4)), 48);;
        assert_eq(|_|"struct_capture", struct_capture(P { fst : 10, snd : 3 }), 21);;
        let tbl = Array::from_map(4, |i| i + 1);
        assert_eq(|_|"match_boxed_capture", match_boxed_capture(Option::some(tbl)), 11);;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_mode());
}
