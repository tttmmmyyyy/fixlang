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

// A cycle of six globals that fix each other round-robin. Each lifted function is deduplicated per
// global, so the `run` fixpoint terminates instead of lifting without end. Each hop adds one, so
// `fix(g0)(20)` counts down to 0 and yields 20, which also checks the rewrite preserves the value.
#[test]
fn test_mutual_global_fix_cycle_terminates() {
    let source = r#"
    module Main;

    g0 : (I64 -> I64) -> I64 -> I64;
    g0 = |self, x| if x <= 0 { 0 } else { 1 + fix(g1)(x - 1) };
    g1 : (I64 -> I64) -> I64 -> I64;
    g1 = |self, x| if x <= 0 { 0 } else { 1 + fix(g2)(x - 1) };
    g2 : (I64 -> I64) -> I64 -> I64;
    g2 = |self, x| if x <= 0 { 0 } else { 1 + fix(g3)(x - 1) };
    g3 : (I64 -> I64) -> I64 -> I64;
    g3 = |self, x| if x <= 0 { 0 } else { 1 + fix(g4)(x - 1) };
    g4 : (I64 -> I64) -> I64 -> I64;
    g4 = |self, x| if x <= 0 { 0 } else { 1 + fix(g5)(x - 1) };
    g5 : (I64 -> I64) -> I64 -> I64;
    g5 = |self, x| if x <= 0 { 0 } else { 1 + fix(g0)(x - 1) };

    cycle_res : I64;
    cycle_res = fix(g0)(20);

    main : IO ();
    main = (
        assert_eq(|_|"cycle web", cycle_res, 20);;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_mode());
}

// A saturated tail self-call buried below a `let` chain inside a `match` arm still lowers to a direct
// self-call, so it runs in constant stack.
#[test]
fn test_deep_tail_position_fix_runs_in_constant_stack() {
    if should_skip_at_none() {
        return;
    }
    let source = r#"
    module Main;

    count : I64 -> (I64, Array I64);
    count = |n| (
        let go = fix(|go, i, arr| (
            if i >= n { (i, arr) };
            let j = i + 1;
            match Option::some(j) {
                some(k) => (
                    let m = k;
                    go(m, arr)
                ),
                none() => (i, arr)
            }
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

// A `fix` whose recursion takes three arguments (four with the capture struct) lowers each self-call
// to a direct call, so it runs in constant stack.
#[test]
fn test_three_arg_fix_runs_in_constant_stack() {
    if should_skip_at_none() {
        return;
    }
    let source = r#"
    module Main;

    count : I64 -> (I64, Array I64);
    count = |n| (
        let go = fix(|go, i, acc, arr| (
            if i >= n { (acc, arr) };
            go(i + 1, acc + 1, arr)
        ));
        go(0, 0, Array::empty(0))
    );

    main : IO ();
    main = (
        let (acc, arr) = count(1000000);
        assert_eq(|_|"unexpected result", acc + arr.@size, 1000000);;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_mode());
}

// Two nested `fix`es where the inner one carries the deep recursion. The outer entry folds into the
// inner and the inner self-call stays direct, so it runs in constant stack.
#[test]
fn test_nested_fix_inner_deep_runs_in_constant_stack() {
    if should_skip_at_none() {
        return;
    }
    let source = r#"
    module Main;

    count : I64 -> (I64, Array I64);
    count = |n| (
        let outer = fix(|outer, seed| (
            let inner = fix(|inner, i, arr| (
                if i >= n { (i, arr) };
                inner(i + 1, arr)
            ));
            inner(seed, Array::empty(0))
        ));
        outer(0)
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

// Captures of exotic value types, an empty capture, `self` used both as a tail self-call and as a
// bare value, a polymorphic `fix` instantiated at two types, and a `fix` inside an iterator fold.
// Each must compute the same value the closure form does. It runs at every optimization level.
#[test]
fn test_fix_exotic_captures_and_dual_use_self_compute_correctly() {
    let source = r#"
    module Main;

    type Cfg = unbox struct { base : I64, step : I64 };
    type Choice = union { add : I64, mul : I64 };

    applyit : (I64 -> I64) -> I64 -> I64;
    applyit = |h, x| h(x);

    empty_cap : I64;
    empty_cap = (fix(|self, x| if x <= 0 { 7 } else { self(x - 1) + 1 }))(5);

    dual_self : I64 -> I64;
    dual_self = |n| (
        (fix(|self, x|
            if x <= 0 { 0 };
            if x % 2 == 0 { self(x - 1) } else { applyit(self, x - 1) + 1 }
        ))(n)
    );

    cap_fun : (I64 -> I64) -> I64 -> I64;
    cap_fun = |transform, n| (
        (fix(|self, x, acc| if x <= 0 { acc } else { self(x - 1, acc + transform(x)) }))(n, 0)
    );

    cap_struct : Cfg -> I64 -> I64;
    cap_struct = |cfg, n| (
        (fix(|self, x, acc| if x <= 0 { acc } else { self(x - 1, acc + cfg.@base + x * cfg.@step) }))(n, 0)
    );

    cap_union : Choice -> I64 -> I64;
    cap_union = |ch, n| (
        (fix(|self, x, acc|
            if x <= 0 { acc };
            self(x - 1, if ch.is_add { acc + ch.as_add } else { acc * ch.as_mul })
        ))(n, 1)
    );

    cap_tuple : (I64, I64, I64) -> I64 -> I64;
    cap_tuple = |t, n| (
        let (a, b, c) = t;
        (fix(|self, x, acc| if x <= 0 { acc } else { self(x - 1, acc + a * x + b - c) }))(n, 0)
    );

    sum_via : [a : Add] Array a -> a -> a;
    sum_via = |tbl, zero| (
        (fix(|self, i, acc| if i >= tbl.@size { acc } else { self(i + 1, acc + tbl.@(i)) }))(0, zero)
    );

    fix_in_fold : I64 -> I64;
    fix_in_fold = |n| (
        Iterator::range(0, n).fold(0, |k, acc|
            acc + (fix(|self, j| if j <= 0 { 0 } else { k + self(j - 1) }))(3)
        )
    );

    main : IO ();
    main = (
        assert_eq(|_|"empty_cap", empty_cap, 12);;
        assert_eq(|_|"dual_self", dual_self(21), 11);;
        assert_eq(|_|"cap_fun", cap_fun(|y| y * y, 5), 55);;
        assert_eq(|_|"cap_struct", cap_struct(Cfg { base : 10, step : 2 }, 4), 60);;
        assert_eq(|_|"cap_union_add", cap_union(Choice::add(3), 4), 13);;
        assert_eq(|_|"cap_union_mul", cap_union(Choice::mul(2), 4), 16);;
        assert_eq(|_|"cap_tuple", cap_tuple((2, 5, 1), 3), 24);;
        let ints : Array I64 = [1, 2, 3, 4];
        assert_eq(|_|"sum_via_int", sum_via(ints, 0), 10);;
        let flts : Array F64 = [1.5, 2.5, 3.0];
        assert_eq(|_|"sum_via_flt", sum_via(flts, 0.0), 7.0);;
        assert_eq(|_|"fix_in_fold", fix_in_fold(5), 30);;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_mode());
}

// `self` and boxed captures crossing the direct self-call in demanding ways: `self` escaping into a
// returned closure and into a returned boxed struct field called after the `fix` returns, a
// triple-nested `fix` whose innermost closes over both outer self-references and a boxed capture, a
// boxed state plus an array threaded through and returned together, and two `fix` expressions sharing
// one boxed capture. `develop_mode` runs this under valgrind, so an unbalanced retain/release shows.
#[test]
fn test_fix_self_escape_and_boxed_captures_are_memory_safe() {
    let source = r#"
    module Main;

    type Holder = box struct { fn : I64 -> I64, tag : I64 };
    type State = box struct { acc : I64, cnt : I64 };

    escape_closure : Array I64 -> (I64 -> I64);
    escape_closure = |tbl| (
        (fix(|self, x|
            if x <= 0 { |z| z + tbl.@(0) };
            |z| self(x - 1)(z) + tbl.@(x % tbl.@size)
        ))(3)
    );

    escape_struct : Array I64 -> Holder;
    escape_struct = |tbl| (
        (fix(|self, x|
            if x <= 0 { Holder { fn : |z| z + tbl.@(0), tag : 0 } };
            Holder { fn : |z| (self(x - 1).@fn)(z) + tbl.@(x % tbl.@size), tag : x }
        ))(3)
    );

    triple_nested : Array I64 -> I64;
    triple_nested = |tbl| (
        (fix(|a, x|
            if x <= 0 { tbl.@(0) };
            (fix(|b, y|
                if y <= 0 { a(x - 1) };
                (fix(|c, z|
                    if z <= 0 { b(y - 1) + tbl.@(z % tbl.@size) };
                    c(z - 1) + tbl.@(z % tbl.@size)
                ))(2)
            ))(2)
        ))(3)
    );

    state_array : Array I64 -> I64 -> (State, Array I64);
    state_array = |seed, depth| (
        (fix(|go, i, st, arr|
            if i >= depth { (st, arr) };
            let st = State { acc : st.@acc + seed.@(i % seed.@size), cnt : st.@cnt + 1 };
            go(i + 1, st, arr.push_back(i))
        ))(0, State { acc : 0, cnt : 0 }, Array::empty(0))
    );

    two_share : Array I64 -> I64;
    two_share = |tbl| (
        let a = (fix(|self, x| if x <= 0 { 0 } else { tbl.@(x % tbl.@size) + self(x - 1) }))(6);
        let b = (fix(|self, x| if x <= 0 { 0 } else { tbl.@(x % tbl.@size) * 2 + self(x - 1) }))(6);
        a + b + tbl.@(0)
    );

    main : IO ();
    main = (
        let tbl = Array::from_map(4, |i| i + 1);
        let f = escape_closure(tbl);
        assert_eq(|_|"escape_closure", f(10) + f(20) + tbl.@(3), 54);;
        let h = escape_struct(tbl);
        assert_eq(|_|"escape_struct", (h.@fn)(5) + h.@tag + tbl.@(1), 20);;
        let tbl3 = Array::from_map(3, |i| i + 1);
        assert_eq(|_|"triple_nested", triple_nested(tbl3) + tbl3.@(2), 40);;
        let seed = Array::from_map(4, |i| i + 1);
        let (st, arr) = state_array(seed, 50);
        assert_eq(|_|"state_array", st.@acc + st.@cnt + arr.@size + seed.@(0), 224);;
        assert_eq(|_|"two_share", two_share(tbl3), 37);;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_mode());
}
