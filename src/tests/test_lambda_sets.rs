use crate::{
    configuration::Configuration,
    constants::MAX_LS_UNION_VARIANTS,
    tests::test_util::{
        assert_source_prints, build_within_and_run, build_within_instructions_and_run,
        run_source_capture, test_source,
    },
};
use std::time::Duration;

// Programs whose function values take the paths the lambda set inference follows: through a
// recursion, into a value that captures a value of its own set, through a `fix` left as a closure,
// and through the fields and the state of a loop. Most run under the development configuration,
// where the inference runs and valgrind checks the result.

/// A recursion that passes itself a lambda wrapping the function it was given: the lambda at depth
/// `n` captures the one at depth `n - 1`.
#[test]
fn test_a_recursion_wrapping_its_function_computes_the_wrapped_chain() {
    let source = r#"
    module Main;

    rec : (I64 -> I64) -> I64 -> I64;
    rec = |f, n| if n == 0 { f(0) } else { rec(|x| f(x) * 2, n - 1) };

    main : IO ();
    main = (
        let k = *get_args.map(|args| args.@size);
        assert_eq(|_|"unexpected result", rec(|x| x + k, 3), 8 * k);;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_mode());
}

/// Folding an array of functions into their composition: the state captures the state of the step
/// before, as long a chain as the array.
#[test]
fn test_a_fold_composing_functions_applies_them_in_order() {
    let source = r#"
    module Main;

    compose_all : Array (I64 -> I64) -> I64 -> I64;
    compose_all = |fs| fs.to_iter.fold(|x| x, |f, acc| |x| f(acc(x)));

    main : IO ();
    main = (
        let k = *get_args.map(|args| args.@size);
        let fs = [|x| x + k, |x| x * 2, |x| x - 1];
        assert_eq(|_|"unexpected result", compose_all(fs)(3), (3 + k) * 2 - 1);;
        let fs = Iterator::range(0, 100).fold([], |i, fs| fs.push_back(|x| x + i));
        assert_eq(|_|"unexpected result", compose_all(fs)(0), 4950);;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_mode());
}

/// A `fix` whose argument reaches it through a parameter, so that `defunctionalize_fix` leaves the
/// combinator in place.
#[test]
fn test_a_fix_given_its_function_through_a_parameter_recurses() {
    let source = r#"
    module Main;

    run_fixed : ((I64 -> I64) -> I64 -> I64) -> I64 -> I64;
    run_fixed = |step, n| fix(step)(n);

    main : IO ();
    main = (
        let k = *get_args.map(|args| args.@size);
        let sum = run_fixed(|loop, i| if i == 0 { 0 } else { k * i + loop(i - 1) }, 10);
        assert_eq(|_|"unexpected result", sum, 55 * k);;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_mode());
}

/// A lambda the callee chooses in place of the one it was given, joining it at an `if`.
#[test]
fn test_a_lambda_the_callee_chooses_over_the_given_one_runs() {
    let source = r#"
    module Main;

    apply_or_zero : (I64 -> I64) -> I64 -> I64;
    apply_or_zero = |f, x| (
        let g = if x < 0 { |_| 0 } else { f };
        g(x)
    );

    main : IO ();
    main = (
        let k = *get_args.map(|args| args.@size);
        assert_eq(|_|"unexpected result", apply_or_zero(|x| x + k, 1), 1 + k);;
        assert_eq(|_|"unexpected result", apply_or_zero(|x| x + k, -1), 0);;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_mode());
}

/// A struct carrying its operation in a field, rebuilt by a setter, chosen at an `if` and carried
/// as the state of a loop, as a segment tree carries its monoid.
#[test]
fn test_a_function_in_a_struct_field_survives_the_state_of_a_loop() {
    let source = r#"
    module Main;

    type Acc = box struct { op : (I64, I64) -> I64, total : I64 };

    add_to : I64 -> Acc -> Acc;
    add_to = |x, acc| acc.set_total((acc.@op)((acc.@total, x)));

    main : IO ();
    main = (
        let k = *get_args.map(|args| args.@size);
        let acc = Acc { op : |(x, y)| x + y + k, total : 0 };
        let acc = Iterator::range(0, 10).fold(acc, |i, acc| (
            if i % 2 == 0 { add_to(i, acc) } else { acc }
        ));
        assert_eq(|_|"unexpected result", acc.@total, 20 + 5 * k);;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_mode());
}

/// Two recursions handing the next round a lambda wrapping the function they were given, one on its
/// own and one around a cycle of two functions: the set of the function they take holds itself.
#[test]
fn test_recursions_wrapping_their_function_on_each_round_compute_the_wrapped_chain() {
    let source = r#"
    module Main;

    grow : I64 -> (I64 -> I64) -> I64;
    grow = |n, op| (
        if n == 0 { op(0) };
        grow(n - 1, |x| op(x) + n)
    );

    tick : I64 -> (I64 -> I64) -> I64;
    tick = |n, op| (
        if n == 0 { op(3) };
        tock(n - 1, |x| op(x) + 1)
    );
    tock : I64 -> (I64 -> I64) -> I64;
    tock = |n, op| (
        if n == 0 { op(4) };
        tick(n - 1, op)
    );

    main : IO ();
    main = (
        assert_eq(|_|"unexpected result", grow(4, |x| x) + tick(4, |x| x * 100), 312);;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_mode());
}

/// A function taking four functions, whose recursive calls each wrap a different one of them and
/// pass the rest through, so that the four sets grow independently.
#[test]
fn test_a_recursion_wrapping_one_of_four_functions_on_each_call_computes_them_all() {
    let source = r#"
    module Main;

    g : (I64 -> I64) -> (I64 -> I64) -> (I64 -> I64) -> (I64 -> I64) -> I64 -> I64;
    g = |p0, p1, p2, p3, n| (
        if n <= 0 { (p0(0) + p1(0) + p2(0) + p3(0)) % 1000 };
        ( g(|x| (p0(x) + 1) % 97, p1, p2, p3, n - 1)
        + g(p0, |x| (p1(x) + 2) % 97, p2, p3, n - 1)
        + g(p0, p1, |x| (p2(x) + 3) % 97, p3, n - 1)
        + g(p0, p1, p2, |x| (p3(x) + 4) % 97, n - 1) ) % 1000
    );

    main : IO ();
    main = (
        let k = 3;
        let answer = g(|x| (x * k + 0) % 97, |x| (x * k + 1) % 97, |x| (x * k + 2) % 97,
            |x| (x * k + 3) % 97, 2);
        assert_eq(|_|"unexpected result", answer, 176);;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_mode());
}

/// A function handing the next round the function it was given wrapped in one of four lambdas, so
/// that the set holds the wrappers in every order.
#[test]
fn test_a_recursion_wrapping_its_function_four_ways_computes_every_order() {
    let source = r#"
    module Main;

    g : (I64 -> I64) -> I64 -> I64;
    g = |op, n| (
        if n <= 0 { op(0) % 97 };
        ( g(|x| (op(x) + 1) % 97, n - 1)
        + g(|x| (op(x) + 2) % 97, n - 1)
        + g(|x| (op(x) + 3) % 97, n - 1)
        + g(|x| (op(x) + 4) % 97, n - 1) ) % 1000
    );

    main : IO ();
    main = (
        let k = 3;
        assert_eq(|_|"unexpected result", g(|x| (x * k) % 97, 3), 480);;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_mode());
}

/// Two functions calling each other, each round swapping which of its two functions goes in which
/// parameter while wrapping one of them, and calling the other twice through a third function.
#[test]
fn test_two_functions_swapping_places_around_a_cycle_compute_both() {
    let source = r#"
    module Main;

    apply_twice : (I64 -> I64) -> I64 -> I64;
    apply_twice = |op, n| (
        if n <= 0 { 0 };
        op(op(n)) + apply_twice(op, n - 1)
    );

    ping : (I64 -> I64) -> (I64 -> I64) -> I64 -> I64;
    ping = |p, q, n| (
        if n <= 0 { p(1) + q(2) };
        (apply_twice(p, n) + pong(q, |x| (p(x) + 1) % 97, n - 1)) % 99991
    );

    pong : (I64 -> I64) -> (I64 -> I64) -> I64 -> I64;
    pong = |p, q, n| (
        if n <= 0 { p(3) + q(4) };
        (apply_twice(q, n) + ping(|x| (q(x) + 2) % 97, p, n - 1)) % 99991
    );

    main : IO ();
    main = (
        let k = 11;
        assert_eq(|_|"unexpected result", ping(|x| (x * k) % 97, |x| (x + k) % 97, 5), 938);;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_mode());
}

/// A lambda given to `fold` capturing four functions whose sets grow independently, each round of
/// the recursion around it wrapping a different one.
#[test]
fn test_a_lambda_capturing_four_growing_functions_calls_them_all() {
    let source = r#"
    module Main;

    terminal : (I64 -> I64) -> I64 -> I64;
    terminal = |op, n| ( if n <= 0 { 0 }; op(n) + terminal(op, n - 1) );

    relay : (I64 -> I64) -> (I64 -> I64) -> (I64 -> I64) -> (I64 -> I64) -> I64 -> I64;
    relay = |p0, p1, p2, p3, n| (
        if n <= 0 { 0 };
        Iterator::range(0, n).fold(0, |i, acc| (
            acc + terminal(p0, i) + terminal(p1, i) + terminal(p2, i) + terminal(p3, i)
        )) % 1000
        + ( relay(|x| (p0(x) + 1) % 97, p1, p2, p3, n - 1)
          + relay(p0, |x| (p1(x) + 2) % 97, p2, p3, n - 1)
          + relay(p0, p1, |x| (p2(x) + 3) % 97, p3, n - 1)
          + relay(p0, p1, p2, |x| (p3(x) + 4) % 97, n - 1) ) % 1000
    );

    main : IO ();
    main = (
        let k = 3;
        let answer = relay(|x| (x*k+0)%97, |x| (x*k+1)%97, |x| (x*k+2)%97, |x| (x*k+3)%97, 2);
        assert_eq(|_|"unexpected result", answer, 18);;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_mode());
}

/// A cycle of three functions, one turn of which wraps the function it carries one level deeper and
/// hands it to `fold` as well.
#[test]
fn test_a_cycle_of_three_wrapping_its_function_on_each_turn_computes_the_chain() {
    let source = r#"
    module Main;

    terminal : (I64 -> I64) -> I64 -> I64;
    terminal = |op, n| ( if n <= 0 { 0 }; op(n) + terminal(op, n - 1) );

    a : (I64 -> I64) -> I64 -> I64;
    a = |op, n| (
        if n <= 0 { op(1) };
        let wrapped = |x| (op(x) + 1) % 97;
        Iterator::range(0, n).fold(0, |i, acc| acc + terminal(wrapped, i)) % 1000
            + b(wrapped, n - 1)
    );
    b : (I64 -> I64) -> I64 -> I64;
    b = |op, n| c(op, n) + terminal(op, n);
    c : (I64 -> I64) -> I64 -> I64;
    c = |op, n| a(op, n) + terminal(op, n);

    main : IO ();
    main = (
        let k = 3;
        assert_eq(|_|"unexpected result", a(|x| (x * k) % 97, 6), 569);;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_mode());
}

/// A recursion handing the next round a lambda that captures, in an `Option`, the function it was
/// given: the set of that function holds the one lambda, and its capture struct would hold in place
/// a capture struct of the same set.
#[test]
fn test_a_lambda_capturing_an_optional_function_of_its_own_set_computes_the_chain() {
    let source = r#"
    module Main;

    chain : I64 -> Option (I64 -> I64) -> I64 -> I64;
    chain = |n, prev, x| (
        let run = |y| if prev.is_some { (prev.as_some)(y) } else { y };
        if n == 0 { run(x) };
        chain(n - 1, some(|y| run(y) + n), x)
    );

    main : IO ();
    main = (
        let k = *get_args.map(|args| args.@size);
        assert_eq(|_|"unexpected result", chain(3, none(), k), k + 6);;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_mode());
}

/// The lambda `mkm` returns is built both in a recursion that carries it in an `Option` and in a
/// direct call, and each computes its result. The set of the captured `g` reaches itself in the
/// recursion's copy, where the lambda's parameter is given a set of closures, and holds one lambda
/// in the direct call's, where the parameter is given a union of two lambdas.
#[test]
fn test_a_lambda_built_both_in_a_cycle_and_out_of_one_captures_each_its_own_way() {
    let source = r#"
    module Main;

    mkm : I64 -> Option (I64 -> I64) -> (I64 -> I64);
    mkm = |d, o| if d < -1000000 { mkm(d + 1, o) } else { (
        let g = |y| match o { some(f) => f(y) + 1, none(_) => y };
        |x| g(x) * 2 + g(x + 1)
    ) };

    go : I64 -> Option (I64 -> I64) -> I64 -> I64;
    go = |n, o, x| if n == 0 { mkm(n, o)(x) } else { go(n - 1, some(mkm(n, o)), x) };

    main : IO ();
    main = (
        let k = *get_args.map(|args| args.@size);
        let a = go(k, none(), 1);
        let b = mkm(k, some(if k > 2 { |x| x + 1 } else { |x| x * 3 }))(5);
        assert_eq(|_|"unexpected result", (a, b), (18, 51));;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_mode());
}

/// The source of the tests of a lambda capturing `g`, a function `mk2` chooses from two of its
/// parameters: `MK2_CALLS` is replaced by the two calls of `mk2`. One call gives both parameters
/// one union of two lambdas, so that `g`'s set holds one lambda, and the other gives them two
/// unions, so that it holds two.
const CHOSEN_FROM_TWO_SOURCE: &str = r#"
    module Main;

    wrap : I64 -> (I64 -> I64) -> (I64 -> I64);
    wrap = |d, f| if d < -1000000 { wrap(d + 1, f) } else { |x| f(x) + d };

    mk2 : (I64 -> I64) -> (I64 -> I64) -> (I64 -> I64) -> I64 -> (I64 -> I64);
    mk2 = |r, p, q, d| if d < -1000000 { mk2(r, p, q, d + 1) } else { (
        let g = if r(d) > 3 { wrap(d, p) } else { wrap(d, q) };
        |x| g(x) * 2 + g(x + 1)
    ) };

    main : IO ();
    main = (
        let k = *get_args.map(|args| args.@size);
        let s1 = if k > 2 { |x| x + 1 } else { |x| x * 3 };
        let s2 = if k > 3 { |x| x + 5 } else { |x| x * 7 };
        MK2_CALLS
        assert_eq(|_|"unexpected result", (a, b), (15, 24));;
        pure()
    );
    "#;

/// The lambda `mk2` returns computes both results when it is first built where `g`'s set holds one
/// lambda.
#[test]
fn test_a_lambda_capturing_a_function_chosen_from_one_set_then_two_computes_both() {
    let source = CHOSEN_FROM_TWO_SOURCE.replace(
        "MK2_CALLS",
        "let a = mk2(|x| x + k, s1, s1, k)(1); let b = mk2(s2, s1, s2, k)(2);",
    );
    test_source(&source, Configuration::develop_mode());
}

/// The lambda `mk2` returns computes both results when it is first built where `g`'s set holds two
/// lambdas.
#[test]
fn test_a_lambda_capturing_a_function_chosen_from_two_sets_then_one_computes_both() {
    let source = CHOSEN_FROM_TWO_SOURCE.replace(
        "MK2_CALLS",
        "let b = mk2(s2, s1, s2, k)(2); let a = mk2(|x| x + k, s1, s1, k)(1);",
    );
    test_source(&source, Configuration::develop_mode());
}

/// A lambda capturing `g`, a function `k` chooses from its two function parameters, computes both
/// results when `k` is called once with one function for both and once, through `w2`, with two
/// functions that `wrap_a` builds over two different sets of two lambdas: `g`'s set holds one
/// lambda in the first copy, and in the second two elements whose captured sets are different
/// unions. The second set is a union of two classes of `mk`'s lambda, one of them the class of the
/// first set. Reaching the second copy through `w2` puts its sets before the first's, so the
/// lambda's class is first met where `g`'s set holds two elements.
#[test]
fn test_a_lambda_capturing_a_function_chosen_from_two_alike_sets_computes_both() {
    let source = r#"
    module Main;

    mk : I64 -> (I64 -> I64) -> I64 -> I64;
    mk = |d, f| if d > 100 { mk(d - 1, f) } else { |x| f(x) + 1 };

    wrap_a : I64 -> (I64 -> I64) -> I64 -> I64;
    wrap_a = |d, h| if d > 100 { wrap_a(d - 1, h) } else { |x| h(x) * 2 };

    k : I64 -> (I64 -> I64) -> (I64 -> I64) -> I64 -> I64;
    k = |d, f1, f2| if d > 100 { k(d - 1, f1, f2) } else {
        let g = if d == 1 { mk(d, f1) } else { mk(d, f2) };
        |y| g(y) + d
    };

    w2 : I64 -> (I64 -> I64) -> (I64 -> I64) -> I64 -> I64;
    w2 = |d, a, b| if d > 100 { w2(d - 1, a, b) } else { k(d, a, b) };

    main : IO ();
    main = (
        let n = *get_args.map(|args| args.@size);
        let h1 = if n > 1 { |x| x + n } else { |x| x * n };
        let h2 = if n > 2 { |x| x + 10 * n } else { |x| x - n };
        let a1 = wrap_a(n, h1);
        let a2 = wrap_a(n, h2);
        let r2 = w2(n, a1, a2)(5);
        let r1 = k(n, a1, a1)(5);
        assert_eq(|_|"unexpected result", (r1, r2), (12, 12));;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_mode());
}

/// An array of one lambda more than `MAX_LS_UNION_VARIANTS`, as Fix source: the `i`-th computes
/// `x {op} n * i` of its argument `x` and a captured `n`. Every lambda is a class of its own, so the
/// set of the array's elements is represented by closures.
fn array_of_too_many_lambdas(op: &str) -> String {
    let lambdas: Vec<String> = (0..MAX_LS_UNION_VARIANTS + 1)
        .map(|i| format!("|x| x {} n * {}", op, i))
        .collect();
    format!("[{}]", lambdas.join(", "))
}

/// A recursion carrying in an `Option` a function `mk` builds from one of two functions, each read
/// from an array of its own of `array_of_too_many_lambdas`, so that the sets of the two are
/// different sets represented by closures: the set of the carried function holds `mk`'s lambda
/// twice, two elements of one class, and each captures in place, through the `Option`, a function
/// of that set again.
#[test]
fn test_two_elements_of_one_class_capturing_an_optional_function_of_their_own_set_compute_the_chain(
) {
    let source = format!(
        r#"
    module Main;

    mk : I64 -> (I64 -> I64) -> Option (I64 -> I64) -> Option (I64 -> I64);
    mk = |d, g, o| if d > 100 {{ mk(d - 1, g, o) }} else {{
        some(|y| g(y) + match o {{ some(f) => f(y), none(_) => 0 }})
    }};

    rec : I64 -> (I64 -> I64) -> (I64 -> I64) -> Option (I64 -> I64) -> I64 -> I64;
    rec = |n, g1, g2, o, x| if n == 0 {{ match o {{ some(f) => f(x), none(_) => x }} }} else {{
        let o2 = if n % 2 == 0 {{ mk(n, g1, o) }} else {{ mk(n, g2, o) }};
        rec(n - 1, g1, g2, o2, x)
    }};

    main : IO ();
    main = (
        let n = *get_args.map(|args| args.@size);
        let g1 = {}.@(n);
        let g2 = {}.@(n + 1);
        assert_eq(|_|"unexpected result", rec(4, g1, g2, none(), 3), 10);;
        pure()
    );
    "#,
        array_of_too_many_lambdas("+"),
        array_of_too_many_lambdas("-")
    );
    test_source(&source, Configuration::develop_mode());
}

/// Two folds handing `mk` each lambda of an array of more lambdas than `MAX_LS_UNION_VARIANTS`, the
/// two arrays holding different lambdas: `mk`'s lambda captures a value of one array's set in one
/// copy and of the other's in the other. Both arrays' sets are represented by closures, and so are
/// alike to the lambda that captures them, and each fold computes its sum.
#[test]
fn test_a_lambda_capturing_either_of_two_sets_of_too_many_classes_computes_both() {
    let count = MAX_LS_UNION_VARIANTS + 1;
    let source = format!(
        r#"
    module Main;

    mk : I64 -> (I64 -> I64) -> (I64 -> I64);
    mk = |d, f| if d < -1000000 {{ mk(d + 1, f) }} else {{ |x| f(x) + 1 }};

    main : IO ();
    main = (
        let n = *get_args.map(|args| args.@size);
        let fs = {};
        let gs = {};
        let a = fs.to_iter.fold(0, |f, acc| mk(n, f)(acc));
        let b = gs.to_iter.fold(0, |g, acc| mk(n, g)(acc));
        assert_eq(|_|"unexpected result", (a, b), ({} * n + {}, -{} * n + {}));;
        pure()
    );
    "#,
        array_of_too_many_lambdas("+"),
        array_of_too_many_lambdas("-"),
        count * (count - 1) / 2,
        count,
        count * (count - 1) / 2,
        count
    );
    test_source(&source, Configuration::develop_mode());
}

/// A program whose `g` returns one of more lambdas than `MAX_LS_UNION_VARIANTS`, so that the
/// function it returns is a closure. Its `main` binds `f` and `f2` to two different lambdas, then
/// runs `calls`, which bind `r1` and `r2`, and asserts that they are 16 and 38. The field `fun` of
/// the struct `Holder` has the type `fun_ty`.
fn returned_closure_source(fun_ty: &str, calls: &str) -> String {
    let branches: Vec<String> = (0..MAX_LS_UNION_VARIANTS + 1)
        .map(|i| format!("if k == {} {{ |x| x * {} + {} }} else", i, i + 2, i))
        .collect();
    format!(
        r#"
    module Main;

    type Holder = struct {{ fun : {}, tag : I64 }};

    g : (I64 -> I64) -> I64 -> I64;
    g = |a| (
        let k = a(0);
        {} {{ |x| x - 1 }}
    );

    main : IO ();
    main = (
        let n = *get_args.map(|args| args.@size);
        let f = |y| y + n;
        let f2 = |y| y * n + 3;
        {}
        assert_eq(|_|"unexpected result", (r1, r2), (16, 38));;
        pure()
    );
    "#,
        fun_ty,
        branches.join("\n        "),
        calls
    )
}

/// `g` returns one of more lambdas than `MAX_LS_UNION_VARIANTS`, so the function it returns is a
/// closure. One call stores `g(f)` in a struct field and applies what it reads back, and the other
/// applies `g(f2)` at once, `f` and `f2` being different lambdas. The two calls reach two uncurried
/// functions of `g`, each specialized to the lambda set of its own argument, and each computes its
/// value.
#[test]
fn test_a_returned_closure_read_back_from_a_struct_and_one_applied_at_once_compute_both() {
    let source = returned_closure_source(
        "I64 -> I64",
        r#"let p = Holder { fun : g(f), tag : 1 };
        let r1 = (p.@fun)(5);
        let r2 = g(f2)(7);"#,
    );
    test_source(&source, Configuration::develop_mode());
}

/// `g` returns one of more lambdas than `MAX_LS_UNION_VARIANTS`, so the function it returns is a
/// closure. One call applies `g(f)` at once, and the other reads `g` itself back from a struct
/// field and applies it to `f2`, `f` and `f2` being different lambdas. The direct call reaches the
/// uncurried function of `g` taking two arguments, specialized to the lambda set of `f`. The call
/// through the field reaches another uncurried function of `g` taking two arguments, which the
/// second run of uncurrying defines. Each computes its value.
#[test]
fn test_a_global_read_back_from_a_struct_and_its_uncurried_call_compute_both() {
    test_source(&global_read_back_source(), Configuration::develop_mode());
}

/// Runs `source` under the development configuration, and asserts that it prints `expected_output`
/// and that the body of each global of `globals`, which prints `evaluating <name>` on stderr, runs
/// once.
fn assert_prints_and_evaluates_each_once(source: &str, expected_output: &str, globals: &[&str]) {
    let output = run_source_capture(source, Configuration::develop_mode());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        expected_output
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    for name in globals {
        let marker = format!("evaluating {}", name);
        assert_eq!(
            stderr.lines().filter(|line| *line == marker).count(),
            1,
            "the body of `{}` should run once, and the program printed:\n{}",
            name,
            stderr
        );
    }
}

/// A global array of functions read as it is in one place and extended with other lambdas in two
/// others. Every reference shares the lambda set of its elements, so the program holds one copy of
/// the global, and its body runs once.
#[test]
fn test_a_global_array_of_functions_extended_in_two_places_is_evaluated_once() {
    let source = r#"
    module Main;

    tbl : Array (I64 -> I64);
    tbl = (
        eval debug_eprintln("evaluating tbl");
        let s = Iterator::range(0, 3).fold(0, |i, acc| acc + i);
        [|x| x + s, |x| x * 2]
    );

    main : IO ();
    main = (
        let k = *get_args.map(|args| args.@size) + 4;
        let a = tbl;
        let b = tbl.push_back(|x| x - k);
        let c = tbl.push_back(|x| x * k).push_back(|x| x / 2);
        let (a0, a1, b2, c2, c3) = (a.@(0), a.@(1), b.@(2), c.@(2), c.@(3));
        println((a0(1) + a1(1) + b2(1) + c2(1) + c3(10)).to_string)
    );
    "#;
    assert_prints_and_evaluates_each_once(source, "12", &["tbl"]);
}

/// A global array of functions and a recursive function taking a function, referring to each
/// other: the array's lambda passes a lambda of its own to the function, so the function's argument
/// shares the set of the array's elements, and `main` passing the function lambdas of its own
/// computes them all.
#[test]
fn test_a_global_array_and_a_function_referring_to_each_other_compute_both() {
    let source = r#"
    module Main;

    tbl : Array (I64 -> I64);
    tbl = [|x| apply_first(|y| y + 1, x)];

    apply_first : (I64 -> I64) -> I64 -> I64;
    apply_first = |g, x| (
        if x <= 0 { g(x) }
        else if x % 2 == 0 { apply_first(g, x - 1) }
        else { (tbl.@(0))(x - 1) }
    );

    main : IO ();
    main = (
        let n = *get_args.map(|args| args.@size);
        let a = apply_first(|z| z * 2 + n, 3);
        let b = apply_first(|z| z * 2 + n, 0);
        assert_eq(|_| "", a + b, 2);;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_mode());
}

/// A global array of functions appended to an array of six lambdas of `main`: the set of the
/// global's elements takes in the six, and the program calls all seven.
#[test]
fn test_a_global_array_of_functions_appended_to_a_longer_one_calls_them_all() {
    let source = r#"
    module Main;

    tbl : Array (I64 -> I64);
    tbl = [|x| x + 1];

    main : IO ();
    main = (
        let n = *get_args.map(|args| args.@size);
        let fs = [|x| x + n, |x| x * n, |x| x - n, |x| x / (n + 1), |x| x % (n + 1), |x| x + 2 * n];
        let all = fs.append(tbl);
        assert_eq(|_| "", all.to_iter.fold(0, |f, acc| acc + f(10)), 58);;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_mode());
}

/// A global array of functions, a function it holds, and a function taking a function, the three
/// referring to one another, where the array's body calls the function taking a function outside
/// any of its elements. The array holds the function itself, whose copies depend on the sets every
/// body of the component reaches, and the program computes it. It is built at `-O max`, where `h`
/// stays a function of its own.
#[test]
fn test_a_global_array_holding_a_function_of_its_component_computes_it() {
    let source = r#"
    module Main;

    tbl : Array (I64 -> I64);
    tbl = (
        let z = h(|y| y + 2, 1);
        [f, |x| x + z]
    );

    f : I64 -> I64;
    f = |x| if x <= 0 { 0 } else { (tbl.@(1))(x - 1) + 1 };

    h : (I64 -> I64) -> I64 -> I64;
    h = |k, x| if x < 0 { f(-x) } else if x > 100 { h(k, x - 1) + h(k, x - 2) } else { k(x) };

    main : IO ();
    main = (
        let n = *get_args.map(|args| args.@size);
        let a = (tbl.@(0))(2);
        let b = h(|y| y * 10 + n, 2);
        println((a + b).to_string)
    );
    "#;
    let printed = build_within_and_run(
        source,
        "max",
        Duration::from_secs(600),
        "a global array holding a function of its component",
    );
    assert_eq!(printed, "26");
}

/// A global value and a function taking a function, referring to each other, where `main` calls
/// the function with a lambda of its own: the program holds one copy of the global, and its body
/// runs once.
#[test]
fn test_a_global_value_in_a_component_with_a_function_taking_a_function_is_evaluated_once() {
    let source = r#"
    module Main;

    g : (I64 -> I64) -> I64 -> I64;
    g = |h, x| if x < 0 { v + h(1) } else if x == 0 { h(0) } else { g(h, x - 1) + h(x) };

    v : I64;
    v = (
        eval debug_eprintln("evaluating v");
        g(|y| y + 1, 2)
    );

    main : IO ();
    main = (
        let n = *get_args.map(|args| args.@size);
        let a = g(|y| y * 3 + n, -1);
        println((a + v).to_string)
    );
    "#;
    assert_prints_and_evaluates_each_once(source, "16", &["v"]);
}

/// A function `z` building closures of `w` over two sets that hold the same lambda when `z` is
/// inferred: the pinned set of the global `t`'s elements, and the set of a reference to `incr`.
/// `main`, inferred after `z`, extends the pinned set with a lambda of its own, so the two sets
/// differ in the end, and the copies of `w` building the two closures are given different sets.
#[test]
fn test_closures_over_a_pinned_set_and_one_it_equals_until_extended_compute_both() {
    let source = r#"
    module Main;

    incr : I64 -> I64 = |x| if x <= 0 { 1 } else { incr(x - 1) + 1 };

    t : Array (I64 -> I64) = [incr];

    w : I64 -> (I64 -> I64) -> (I64 -> I64) = |d, f| (
        if d > 0 { w(d - 1, f) } else { |x| f(x) * 2 }
    );

    z : I64 -> Array (I64 -> I64) = |n| (
        if n > 100 { z(n - 1) } else { [w(n, incr), w(n, t.@(0))] }
    );

    main : IO () = (
        let n = *get_args.map(|args| args.@size);
        let u = t.push_back(|x| x - n);
        let a = z(n).to_iter.fold(0, |f, acc| acc * 10 + f(n));
        let b = u.to_iter.fold(0, |f, acc| acc * 10 + f(n));
        assert_eq(|_| "", (a, b), (44, 20));;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_mode());
}

/// The program of `returned_closure_source` in which one call applies `g(f)` at once, and the other
/// reads `g` itself back from a struct field and applies it to `f2` and `7`.
fn global_read_back_source() -> String {
    returned_closure_source(
        "(I64 -> I64) -> I64 -> I64",
        r#"let r1 = g(f)(5);
        let p = Holder { fun : g, tag : 1 };
        let r2 = (p.@fun)(f2)(7);"#,
    )
}

/// How deep the chain of `chain_source` goes in the tests of the bound on the capture structs a
/// value lays out in place.
const CHAIN_DEPTH: usize = 18;

/// What `chain_source()` prints.
const CHAIN_OUTPUT: &str = "98385";

/// A program building a chain of functions `f0` to `f{CHAIN_DEPTH}`, each chosen by an `if` from two
/// lambdas, the lambdas of each capturing the function before, and printing what the last gives.
/// Each set of the chain is of two classes, so a value of `f{i}` represented by a union lays out
/// `2^(i + 2) - 2` capture structs in place: two for `f0`, whose lambdas hold no function, and at
/// each step, for each of the two classes, one plus those of the value before.
fn chain_source() -> String {
    let links: Vec<String> = (1..=CHAIN_DEPTH)
        .map(|i| {
            format!(
                "    let f{i} = if (k + {i}) % 2 == 0 {{ |x| (f{p}(x) + {i}) % 1000003 }} else {{ |x| (f{p}(x) * 3) % 1000003 }};",
                i = i,
                p = i - 1
            )
        })
        .collect();
    format!(
        "module Main;\n\
         \n\
         main : IO ();\n\
         main = (\n\
         \x20   let k = *get_args.map(|args| args.@size);\n\
         \x20   let s = k.to_string;\n\
         \x20   let f0 = if k > 1 {{ |x| x + s.get_size }} else {{ |x| x * 2 }};\n\
         {}\n\
         \x20   println(f{}(k).to_string)\n\
         );\n",
        links.join("\n"),
        CHAIN_DEPTH
    )
}

/// The chain of `chain_source` builds well within a deadline. The first set of the chain whose
/// values would lay out too many capture structs in place is represented by closures; as unions all
/// the way out, the build would take far longer.
#[test]
fn test_a_chain_of_unions_each_capturing_the_one_before_builds_in_time() {
    let printed = build_within_and_run(
        &chain_source(),
        "max",
        Duration::from_secs(300),
        &format!(
            "a chain of {} unions each capturing the one before",
            CHAIN_DEPTH
        ),
    );
    assert_eq!(printed, CHAIN_OUTPUT, "the chain computed a wrong value");
}

/// How many functions the chain of `branching_chain_source` calls through.
const BRANCHING_CHAIN_DEPTH: usize = 14;

/// What `branching_chain_source(BRANCHING_CHAIN_DEPTH)` prints.
const BRANCHING_CHAIN_OUTPUT: &str = "1097390";

/// How the functions of `chain_of_two_lambda_calls_source` take the function and the number.
#[derive(Clone, Copy)]
enum ChainArgs {
    /// As two arguments: `f(g, x)`.
    Curried,
    /// As one tuple: `f((g, x))`.
    Tupled,
}

impl ChainArgs {
    /// The type of a function of the chain returning `result_ty`.
    fn func_ty(self, result_ty: &str) -> String {
        match self {
            ChainArgs::Curried => format!("(I64 -> I64) -> I64 -> {}", result_ty),
            ChainArgs::Tupled => format!("(I64 -> I64, I64) -> {}", result_ty),
        }
    }

    /// The parameters `g` and `x` of a lambda defining a function of the chain.
    fn params(self) -> &'static str {
        match self {
            ChainArgs::Curried => "|g, x|",
            ChainArgs::Tupled => "|(g, x)|",
        }
    }

    /// The call of `func` with the function `g` and the number `x`.
    fn call(self, func: &str, g: &str, x: &str) -> String {
        match self {
            ChainArgs::Curried => format!("{}({}, {})", func, g, x),
            ChainArgs::Tupled => format!("{}(({}, {}))", func, g, x),
        }
    }
}

/// A program calling down a chain of functions `f{depth}` to `f0`, which return `result_ty` and
/// take their arguments as `arg_form` says. Each function calls the next from two places, each with
/// a lambda of its own wrapping the function it was given, and `f0` returns `f0_body`. `globals`
/// holds the definitions of the globals `f0_body` refers to outside the chain. `main` binds `n`
/// and runs `main_body`, where `CHAIN_CALL` stands for the call of `f{depth}` with `|x| x + n` and
/// `n`.
fn chain_of_two_lambda_calls_source(
    depth: usize,
    arg_form: ChainArgs,
    result_ty: &str,
    globals: &str,
    f0_body: &str,
    main_body: &str,
) -> String {
    let func_ty = arg_form.func_ty(result_ty);
    let params = arg_form.params();
    let links: Vec<String> = (1..=depth)
        .map(|i| {
            let next = format!("f{}", i - 1);
            format!(
                "f{i} : {func_ty};\n\
                 f{i} = {params} if x % 3 == 0 {{ {plus} }} else {{ {times} }};",
                i = i,
                func_ty = func_ty,
                params = params,
                plus = arg_form.call(&next, &format!("|y| g(y) + {}", i), "x / 3"),
                times = arg_form.call(&next, &format!("|y| g(y) * {}", i), "x - 1"),
            )
        })
        .collect();
    format!(
        "module Main;\n\
         \n\
         {globals}\
         f0 : {func_ty};\n\
         f0 = {params} {f0_body};\n\
         {links}\n\
         \n\
         main : IO ();\n\
         main = (\n\
         \x20   let n = *(pure(100) : IO I64);\n\
         \x20   {main_body}\n\
         );\n",
        globals = globals,
        func_ty = func_ty,
        params = params,
        f0_body = f0_body,
        links = links.join("\n"),
        main_body = main_body.replace(
            "CHAIN_CALL",
            &arg_form.call(&format!("f{}", depth), "|x| x + n", "n")
        )
    )
}

/// A program calling down a chain of functions `f{depth}` to `f0`, each calling the next from two
/// places, each with a lambda of its own wrapping the function it was given, and printing what the
/// chain computes. Each copy of `f{i}` asks for two copies of `f{i - 1}`, one per lambda, so the
/// copies double at each step.
fn branching_chain_source(depth: usize) -> String {
    chain_of_two_lambda_calls_source(
        depth,
        ChainArgs::Curried,
        "I64",
        "",
        "g(x)",
        "println(CHAIN_CALL.to_string)",
    )
}

/// A bound on the instructions a build of one of the chains below executes: about five times what
/// the build of `branching_chain_source(BRANCHING_CHAIN_DEPTH)` executes with the copy budget
/// (`LS_COPY_BUDGET_BASE`) applied (11.5 billion on x86-64), and a fifth of what it executes with
/// every copy made (286 billion).
const BRANCHING_CHAIN_MAX_INSTRUCTIONS: u64 = 50_000_000_000;

/// The chain of `branching_chain_source(BRANCHING_CHAIN_DEPTH)` builds in fewer than
/// `BRANCHING_CHAIN_MAX_INSTRUCTIONS` instructions. The functions far down the chain go past the
/// copy budget (`LS_COPY_BUDGET_BASE`), so they are given closures, which makes each of them one
/// copy.
/// Where the machine does not let the test count instructions, the test checks the answer alone.
#[test]
fn test_a_chain_of_functions_each_calling_the_next_with_two_lambdas_builds_in_bounded_work() {
    let description = format!(
        "a chain of {} functions each calling the next with two lambdas",
        BRANCHING_CHAIN_DEPTH
    );
    assert_chain_builds_in_bounded_work(
        &branching_chain_source(BRANCHING_CHAIN_DEPTH),
        &description,
        BRANCHING_CHAIN_OUTPUT,
    );
}

/// Builds `source`, a chain `description` names, at the optimization level `max` in fewer than
/// `BRANCHING_CHAIN_MAX_INSTRUCTIONS` instructions (`build_within_instructions_and_run`), and checks
/// that the program prints `expected`.
fn assert_chain_builds_in_bounded_work(source: &str, description: &str, expected: &str) {
    let printed = build_within_instructions_and_run(
        source,
        "max",
        BRANCHING_CHAIN_MAX_INSTRUCTIONS,
        description,
    );
    assert_eq!(printed, expected, "the chain computed a wrong value");
}

/// A depth of the chain of `branching_chain_source` at which a copy for each way a function is
/// asked for would make some 2^24 copies.
const LONG_BRANCHING_CHAIN_DEPTH: usize = 24;

/// What `branching_chain_source(LONG_BRANCHING_CHAIN_DEPTH)` prints.
const LONG_BRANCHING_CHAIN_OUTPUT: &str = "17458065";

/// The chain of `branching_chain_source(LONG_BRANCHING_CHAIN_DEPTH)` builds in fewer than
/// `BRANCHING_CHAIN_MAX_INSTRUCTIONS` instructions (18 billion on x86-64) and computes its value.
/// The functions far down the chain are given closures once they go past the copy budget
/// (`LS_COPY_BUDGET_BASE`), so the copies `specialize` lists stop doubling at each step.
/// Where the machine does not let the test count instructions, the test checks the answer alone.
#[test]
fn test_a_long_chain_of_functions_each_calling_the_next_with_two_lambdas_lists_its_copies_in_bounded_work(
) {
    assert_chain_builds_in_bounded_work(
        &branching_chain_source(LONG_BRANCHING_CHAIN_DEPTH),
        &format!(
            "a chain of {} functions each calling the next with two lambdas",
            LONG_BRANCHING_CHAIN_DEPTH
        ),
        LONG_BRANCHING_CHAIN_OUTPUT,
    );
}

/// What `returning_branching_chain_source(BRANCHING_CHAIN_DEPTH)` prints.
const RETURNING_BRANCHING_CHAIN_OUTPUT: &str = "4361560";

/// The chain of `branching_chain_source`, where each function returns the function it builds in
/// place of calling it: `f0` returns the function it was given, so what `f{i}` returns holds the
/// lambdas `f{i}` and every function after it in the chain build. `main` calls what `f{depth}`
/// returns twice, so that the functions of the chain return a function once their calls are
/// uncurried.
fn returning_branching_chain_source(depth: usize) -> String {
    chain_of_two_lambda_calls_source(
        depth,
        ChainArgs::Curried,
        "(I64 -> I64)",
        "",
        "g",
        "let h = CHAIN_CALL;\n    println((h(n) + h(n + 1)).to_string)",
    )
}

/// The chain of `returning_branching_chain_source(BRANCHING_CHAIN_DEPTH)` builds in fewer than
/// `BRANCHING_CHAIN_MAX_INSTRUCTIONS` instructions (11.7 billion on x86-64) and computes its value.
/// A function far down the chain goes past the copy budget (`LS_COPY_BUDGET_BASE`), so the
/// function it is given is a closure. The chain hands that function back, so every function of the
/// chain returns a closure.
#[test]
fn test_a_chain_of_functions_each_returning_the_next_with_two_lambdas_builds_in_bounded_work() {
    let description = format!(
        "a chain of {} functions each returning the next with two lambdas",
        BRANCHING_CHAIN_DEPTH
    );
    assert_chain_builds_in_bounded_work(
        &returning_branching_chain_source(BRANCHING_CHAIN_DEPTH),
        &description,
        RETURNING_BRANCHING_CHAIN_OUTPUT,
    );
}

/// A depth of the chain of `returning_branching_chain_source` at which what `f{depth}` returns
/// would hold some 2^18 lambdas.
const DEEP_RETURNING_BRANCHING_CHAIN_DEPTH: usize = 18;

/// What `returning_branching_chain_source(DEEP_RETURNING_BRANCHING_CHAIN_DEPTH)` prints.
const DEEP_RETURNING_BRANCHING_CHAIN_OUTPUT: &str = "16826700";

/// The chain of `returning_branching_chain_source(DEEP_RETURNING_BRANCHING_CHAIN_DEPTH)` builds in
/// fewer than `BRANCHING_CHAIN_MAX_INSTRUCTIONS` instructions (14.3 billion on x86-64) and computes its
/// value. What each function returns holds twice the lambdas of what the next returns, so the
/// variable of the result goes past `MAX_LS_VARIABLE_LAMBDAS` within a few steps and is made
/// Dynamic, which stops the doubling.
/// Where the machine does not let the test count instructions, the test checks the answer alone.
#[test]
fn test_a_deep_chain_of_functions_each_returning_the_next_with_two_lambdas_builds_in_bounded_work()
{
    assert_chain_builds_in_bounded_work(
        &returning_branching_chain_source(DEEP_RETURNING_BRANCHING_CHAIN_DEPTH),
        &format!(
            "a chain of {} functions each returning the next with two lambdas",
            DEEP_RETURNING_BRANCHING_CHAIN_DEPTH
        ),
        DEEP_RETURNING_BRANCHING_CHAIN_OUTPUT,
    );
}

/// What `wrapping_branching_chain_source(BRANCHING_CHAIN_DEPTH)` prints.
const WRAPPING_BRANCHING_CHAIN_OUTPUT: &str = "4361562";

/// The chain of `returning_branching_chain_source`, where `f0` hands the function it was given to
/// `wrap`, a function outside the chain, and returns the lambda `wrap` builds around it. `main`
/// calls what `f{depth}` returns twice, so that the functions of the chain return a function once
/// their calls are uncurried. `wrap` and `f0` recurse, which keeps their bodies out of their
/// callers.
fn wrapping_branching_chain_source(depth: usize) -> String {
    chain_of_two_lambda_calls_source(
        depth,
        ChainArgs::Curried,
        "(I64 -> I64)",
        "wrap : (I64 -> I64) -> (I64 -> I64);\n\
         wrap = |f| |y| if y < -1000000 { (wrap(f))(y + 1) } else { f(y) + 1 };\n",
        "if x < -1000000 { f0(g, x + 1) } else { wrap(g) }",
        "let h = CHAIN_CALL;\n    println((h(n) + h(n + 1)).to_string)",
    )
}

/// The chain of `wrapping_branching_chain_source(BRANCHING_CHAIN_DEPTH)` builds in fewer than
/// `BRANCHING_CHAIN_MAX_INSTRUCTIONS` instructions (22.5 billion on x86-64) and computes its value.
/// `f0` goes past the copy budget (`LS_COPY_BUDGET_BASE`), and the lambda `wrap` builds around the
/// function `f0` hands it reaches `main` through the results of the chain: the sets `main` reads
/// that lambda with are the ones the copy of `wrap` under `f0` builds it with.
#[test]
fn test_a_lambda_built_under_a_function_past_its_copy_budget_and_returned_up_a_chain_is_called() {
    assert_chain_builds_in_bounded_work(
        &wrapping_branching_chain_source(BRANCHING_CHAIN_DEPTH),
        &format!(
            "a chain of {} functions each returning the next with two lambdas, the last returning \
             a lambda another function builds",
            BRANCHING_CHAIN_DEPTH
        ),
        WRAPPING_BRANCHING_CHAIN_OUTPUT,
    );
}

/// What lambda set specialization makes of a program, read off the `--emit-rc-ir` dump: a
/// function given a lambda calls it by name, and the lambda's capture becomes a struct the function
/// takes as an argument. The program cannot observe either one — the answer is the same with
/// function values left as closures — so these assert against the dump.
/// A program whose `main` runs `body` with `k` bound to 1, read from the arguments so that the
/// compiler cannot fold a choice `body` makes by it. `body` moves functions through one builtin
/// operation and prints what calling them gives; the operation is the only path from the lambdas
/// it is given to the calls, so the calls reach the right lambdas only when the lambda set flow of
/// the operation declares that path.
fn builtin_flow_source(body: &str) -> String {
    format!(
        r#"
    module Main;

    type B = box struct {{ n : I64, g : I64 -> I64, h : I64 -> I64 }};
    type S = unbox struct {{ n : I64, g : I64 -> I64, h : I64 -> I64 }};
    type U = union {{ num : I64, fa : I64 -> I64, fb : I64 -> I64 }};
    type BU = box union {{ bnum : I64, bfa : I64 -> I64, bfb : I64 -> I64 }};

    main : IO ();
    main = (
        let k = *get_args.map(|args| args.@size);
        {}
        pure()
    );
    "#,
        body
    )
}

/// `mod` on an array of lambdas, which punches one element out, hands it to the modifier and
/// plugs what the modifier returns back: the lambda plugged in calls the one punched out, and
/// the other element is the lambda the array held.
#[test]
fn test_a_function_punched_out_of_an_array_and_plugged_back_wrapped_calls_the_one_it_wraps() {
    let source = builtin_flow_source(
        r#"let fs = [|x| x + k, |x| x * (k + 2)].mod(0, |f| |x| f(x) + 30);
        println(((fs.@(0))(5) * 100 + (fs.@(1))(5)).to_string);;"#,
    );
    assert_source_prints(&source, Configuration::develop_mode(), "3615");
}

/// `mod_fb` on unions holding each variant, with a modifier wrapping the payload: the variant
/// modified holds a lambda calling the one it held, and a union holding another variant comes
/// back holding what it held.
#[test]
fn test_a_function_in_a_union_modified_calls_the_one_it_wraps_and_the_others_stay() {
    let source = builtin_flow_source(
        r#"let us = [fa(|x| x + k), fb(|x| x * (k + 2)), num(k)];
        let us = us.map(|u| u.mod_fb(|f| |x| f(x) + 30));
        println(us.to_iter.fold(0, |u, acc| acc * 100 + (if u.is_fa { (u.as_fa)(5) } else if u.is_fb { (u.as_fb)(5) } else { u.as_num })).to_string);;
        let bu = (if k > 5 { bfa(|x| x + k) } else { bfb(|x| x * (k + 3)) }).mod_bfb(|f| |x| f(x) + 40);
        println((if bu.is_bfa { (bu.as_bfa)(5) } else { (bu.as_bfb)(5) }).to_string);;"#,
    );
    assert_source_prints(&source, Configuration::develop_mode(), "64501\n60");
}

/// A boxed struct built at either arm of an `if`, each arm with lambdas of its own, handed
/// back by `mark_threaded`: its fields are the lambdas of the arm taken.
#[test]
fn test_a_struct_of_functions_marked_threaded_keeps_its_lambdas() {
    let source = builtin_flow_source(
        r#"let b = if k > 5 { B { n : 1, g : |x| x + k, h : |x| x - k } } else { B { n : 2, g : |x| x * (k + 2), h : |x| x + 9 } };
        let b2 = b.mark_threaded;
        println((b2.@g)(5).to_string + " " + (b2.@h)(5).to_string);;"#,
    );
    let mut config = Configuration::develop_mode();
    config.set_threaded();
    assert_source_prints(&source, config, "15 14");
}

#[cfg(test)]
mod dump_tests {
    use super::{
        array_of_too_many_lambdas, chain_source, global_read_back_source, CHAIN_DEPTH, CHAIN_OUTPUT,
    };
    use crate::constants::{
        LS_CAP_PREFIX, LS_COPY_BUDGET_BASE, LS_COPY_SUFFIX, LS_LAM_SUFFIX, LS_UNION_PREFIX,
        MAX_LS_IN_PLACE_CAPTURE_STRUCTS, MAX_LS_STORED_VALUE_WORDS, MAX_LS_UNION_VARIANTS,
    };
    use crate::misc::Set;
    use crate::tests::test_util::{
        build_run_and_read_rc_ir, rc_ir_callee_of, rc_ir_function_bodies,
    };

    /// How many values `pick` of `joined_values_source` chooses among: more than the classes a
    /// union may have (`MAX_LS_UNION_VARIANTS`).
    const JOINED_VALUES: usize = MAX_LS_UNION_VARIANTS + 8;

    /// A program whose `pick` returns one of `JOINED_VALUES` functions, each built by a call of
    /// its own of `mk2` given what a call of its own of `mk` builds. The variable of the result
    /// holds the lambda of `mk2` once per call, each time with a variable of its own for the
    /// function it captures, and every one of those variables holds the lambda of `mk` alone.
    /// `mk` and `mk2` recurse, which keeps their bodies out of `pick`.
    fn joined_values_source() -> String {
        let branches: Vec<String> = (0..JOINED_VALUES)
            .map(|i| format!("    if k == {i} {{ mk2(mk({i})) }} else\n"))
            .collect();
        format!(
            "module Main;\n\
             \n\
             mk : I64 -> (I64 -> I64);\n\
             mk = |n| |x| if x < -1000000 {{ (mk(n))(x + 1) }} else {{ x + n }};\n\
             \n\
             mk2 : (I64 -> I64) -> (I64 -> I64);\n\
             mk2 = |g| |x| if x < -1000000 {{ (mk2(g))(x + 1) }} else {{ g(x) + 1 }};\n\
             \n\
             pick : I64 -> (I64 -> I64) -> (I64 -> I64);\n\
             pick = |k, d| (\n\
             {}\
             \x20   d\n\
             );\n\
             \n\
             main : IO ();\n\
             main = (\n\
             \x20   let n = *(pure(100) : IO I64);\n\
             \x20   let h = pick(n % 7, mk2(mk(1000)));\n\
             \x20   println((h(n) + h(n + 1)).to_string)\n\
             );\n",
            branches.concat()
        )
    }

    /// A variable holding more lambdas than a union may have classes keeps its capture struct
    /// where the lambdas build the same values: `pick` of `joined_values_source` returns the
    /// capture struct of the lambda of `mk2`, and builds no closure. The lambdas of the result's
    /// variable are within `MAX_LS_VARIABLE_LAMBDAS`, and its concrete set holds one element.
    #[test]
    fn test_a_function_returning_one_of_many_values_of_one_lambda_returns_its_capture_struct() {
        let dump = build_run_and_read_rc_ir(
            &joined_values_source(),
            "max",
            "207",
            "a function returning one of many values built by one lambda",
        );
        let picks = rc_ir_function_bodies(&dump, "fn Main::pick");
        assert_eq!(picks.len(), 1, "the functions made of `pick`: {:?}", picks);
        let pick = &picks[0];
        let signature = pick.lines().next().unwrap();
        let (_, result) = signature.rsplit_once(" -> ").unwrap();
        assert!(
            result.contains(&format!("{}@mk2", LS_CAP_PREFIX)),
            "`pick` returns `{}`",
            result
        );
        assert!(!pick.contains("= closure "), "`pick` builds a closure");
    }

    /// The function values that `body`, a function of an RC IR dump, calls: each call whose callee
    /// is a local value. A global is written with its namespace, and a builtin operation's name
    /// carries no `#`, which every local name of the dump does.
    fn closure_calls(body: &str) -> Vec<&str> {
        body.lines()
            .filter_map(rc_ir_callee_of)
            .filter(|callee| callee.contains('#') && !callee.contains("::"))
            .collect()
    }

    /// Whether some function of `dump` named with `name_part` calls a function value through a
    /// closure.
    fn calls_through_a_closure(dump: &str, name_part: &str) -> bool {
        rc_ir_function_bodies(dump, name_part)
            .iter()
            .any(|body| !closure_calls(body).is_empty())
    }

    /// The functions of `dump` named with `name_part`, as `rc_ir_function_bodies` gives them. Fails
    /// the test when there are none.
    fn function_bodies_named(dump: &str, name_part: &str) -> Vec<String> {
        let bodies = rc_ir_function_bodies(dump, name_part);
        assert!(
            !bodies.is_empty(),
            "the dump should hold functions named with `{}`, and it holds none",
            name_part
        );
        bodies
    }

    /// Asserts that `dump` holds functions named with `name_part`, and that none of them builds a
    /// closure or calls one: every function value they call is called by name.
    fn assert_calls_by_name(dump: &str, name_part: &str) {
        for body in function_bodies_named(dump, name_part) {
            assert!(
                !body.contains("= closure ") && closure_calls(&body).is_empty(),
                "every function named with `{}` should call the function values it calls by \
                 name, and this one goes through a closure:\n{}",
                name_part,
                body
            );
        }
    }

    /// How many places of `fan_in_source` call the chain, each with a lambda of its own: more than
    /// `LS_COPY_BUDGET_BASE`, so that the chain stays within its copy budget only by the references
    /// on the way to it.
    const FAN_IN_CALLS: usize = 80;
    const _: () = assert!(FAN_IN_CALLS > LS_COPY_BUDGET_BASE);

    /// How many functions of `fan_in_source` hand the function they were given to the next.
    const FAN_IN_DEPTH: usize = 5;

    /// A program calling a chain of functions `h{FAN_IN_DEPTH}` to `h0` from `FAN_IN_CALLS` places,
    /// each with a lambda of its own, where each `h{i}` hands the function it was given to
    /// `h{i - 1}` and `h0` hands a lambda calling it to `loop`. Every function of the chain is
    /// copied once per place, and so is `loop`, though the program refers to `loop` once.
    fn fan_in_source() -> String {
        let links: Vec<String> = (1..=FAN_IN_DEPTH)
            .map(|i| {
                format!(
                    "h{i} : (I64 -> I64) -> I64 -> I64;\n\
                     h{i} = |g, n| if n < 0 {{ h{i}(g, -n) }} else {{ h{next}(g, n) + {i} }};\n",
                    i = i,
                    next = i - 1
                )
            })
            .collect();
        let calls: Vec<String> = (1..=FAN_IN_CALLS)
            .map(|i| format!("h{}(|x| x + n * {}, n)", FAN_IN_DEPTH, i))
            .collect();
        format!(
            "module Main;\n\
             \n\
             h0 : (I64 -> I64) -> I64 -> I64;\n\
             h0 = |g, n| if n < 0 {{ h0(g, -n) }} else {{ loop((0, 0), |(i, s)| if i == n {{ break $ s }} else {{ continue $ (i + 1, s + g(i)) }}) }};\n\
             {}\
             \n\
             main : IO ();\n\
             main = (\n\
             \x20   let n = *(pure(100) : IO I64);\n\
             \x20   println(({}).to_string)\n\
             );\n",
            links.join(""),
            calls.join(" + ")
        )
    }

    /// Copies that grow with the references on the way to a function stay within the copy budget
    /// (`LS_COPY_BUDGET_BASE`): `fan_in_source` copies each function of its chain and `loop` once per
    /// call of the chain, and every copy of `loop` calls the lambda it was given by name.
    #[test]
    fn test_a_loop_at_the_end_of_a_chain_called_from_many_places_calls_each_lambda_by_name() {
        let dump = build_run_and_read_rc_ir(
            &fan_in_source(),
            "max",
            "32797200",
            "a chain of functions called from many places, handing a lambda to `loop`",
        );
        assert_calls_by_name(&dump, "Std::loop");
    }

    /// How many places of `fan_out_under_a_hub_source` call `a`, and how many places of `a` call
    /// `b`, each with a lambda of its own.
    const FAN_OUT_CALLS: usize = 10;

    /// How many places of `fan_out_under_a_hub_source` call `u`, each with a lambda of its own.
    const HUB_CALLS: usize = 30;

    /// A program whose `main` calls `a` from `FAN_OUT_CALLS` places and `u` from `HUB_CALLS`
    /// places, each with a lambda of its own, where `a` calls `b` from `FAN_OUT_CALLS` places, each
    /// with a lambda of its own wrapping the function it was given. `b` is asked for
    /// `FAN_OUT_CALLS * FAN_OUT_CALLS` bodies, and `u` for one per place.
    fn fan_out_under_a_hub_source() -> String {
        let b_calls: Vec<String> = (1..=FAN_OUT_CALLS)
            .map(|k| format!("b(|y| g(y) + {}, x)", k))
            .collect();
        let a_calls: Vec<String> = (1..=FAN_OUT_CALLS)
            .map(|i| format!("a(|x| x + n * {}, n)", i))
            .collect();
        let u_calls: Vec<String> = (1..=HUB_CALLS)
            .map(|k| format!("u(|x| x + {}, n)", k))
            .collect();
        format!(
            "module Main;\n\
             \n\
             b : (I64 -> I64) -> I64 -> I64;\n\
             b = |g, x| if x < 0 {{ b(g, -x) }} else {{ g(x) }};\n\
             \n\
             a : (I64 -> I64) -> I64 -> I64;\n\
             a = |g, x| {};\n\
             \n\
             u : (I64 -> I64) -> I64 -> I64;\n\
             u = |g, x| if x < 0 {{ u(g, -x) }} else {{ g(x) }};\n\
             \n\
             main : IO ();\n\
             main = (\n\
             \x20   let n = *(pure(100) : IO I64);\n\
             \x20   println(({} + {}).to_string)\n\
             );\n",
            b_calls.join(" + "),
            a_calls.join(" + "),
            u_calls.join(" + ")
        )
    }

    /// The budget of a component counts only the references on the way to it: in
    /// `fan_out_under_a_hub_source`, the `FAN_OUT_CALLS * FAN_OUT_CALLS` bodies `b` is asked for
    /// go past `LS_COPY_BUDGET_BASE` plus the references on the way to `b`: the one from the entry
    /// to `main` and the `2 * FAN_OUT_CALLS` from `main` to `a` and from `a` to `b`. So `b` has one
    /// body, which calls the function it was given through a closure, though the `HUB_CALLS`
    /// references from `main` to `u` would cover the bodies it was asked for. `u`, copied once per
    /// reference to it, calls each of its lambdas by name.
    #[test]
    fn test_references_of_a_hub_to_other_parts_leave_a_fan_out_past_its_copy_budget() {
        let dump = build_run_and_read_rc_ir(
            &fan_out_under_a_hub_source(),
            "max",
            "69015",
            "a fan-out of two steps under a `main` calling another function from many places",
        );
        assert!(
            calls_through_a_closure(&dump, "Main::b"),
            "`b` is asked for more bodies than its budget, and should call the function it was \
             given through a closure"
        );
        assert_calls_by_name(&dump, "Main::u");
    }

    /// Verifies that `main` calls `g` by name both where it applies `g` to its two arguments at once
    /// and where it applies `g`, read back from a struct field, to them. Lambda set specialization
    /// turns the call through the field into `g(f2)(7)` after the first run of uncurrying, and the
    /// second run gives `g` an uncurried function taking both arguments.
    #[test]
    fn test_a_call_of_a_global_read_back_from_a_struct_calls_it_by_name() {
        let dump = build_run_and_read_rc_ir(
            &global_read_back_source(),
            "max",
            "",
            "a global read back from a struct and applied to two arguments",
        );
        assert_calls_by_name(&dump, "Main::main");
    }

    /// The function header of `body`, a function of an RC IR dump: its first line.
    fn header(body: &str) -> &str {
        body.lines().next().unwrap()
    }

    /// The parameter list of a dumped function header, taken from the parenthesis that opens it to
    /// the one that closes it.
    ///
    /// A parameter's own type carries parentheses — a function type is written `(a) -> b` — so the
    /// list runs to the parenthesis that brings the nesting back to where it started.
    fn parameters(header: &str) -> &str {
        let open = header
            .find('(')
            .unwrap_or_else(|| panic!("the function header opens no parameter list:\n{}", header));
        let mut depth = 0;
        for (offset, character) in header[open..].char_indices() {
            match character {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        return &header[open + 1..open + offset];
                    }
                }
                _ => {}
            }
        }
        panic!(
            "the parameter list of the function header stays open:\n{}",
            header
        )
    }

    /// The words of the parameter list of the function header `header`: the names of the parameters
    /// and of the types they and their type arguments have.
    fn parameter_tokens(header: &str) -> impl Iterator<Item = &str> {
        parameters(header)
            .split(|c: char| c.is_whitespace() || c == ',' || c == '(' || c == ')')
            .filter(|token| !token.is_empty())
    }

    /// The capture structs among the parameters of the function header `header`, by the name of
    /// their type.
    fn capture_struct_parameters(header: &str) -> Vec<&str> {
        parameter_tokens(header)
            .filter(|token| token.contains(LS_CAP_PREFIX))
            .collect()
    }

    /// `fold` handed a lambda that captures `n`.
    const FOLD_SOURCE: &str = r#"
    module Main;

    main : IO () = (
        let n = 3;
        println(Iterator::range(0, 10).fold(0, |i, acc| acc + i * n).to_string)
    );
    "#;

    /// What `FOLD_SOURCE` prints: the sum of `3 * i` over `0..9`.
    const FOLD_OUTPUT: &str = "135";

    /// A function given a lambda calls it by name. At `basic`, where the rewrite does not run, the
    /// same fold calls the lambda through a closure, which shows that the assertion can detect a
    /// call through a closure.
    #[test]
    fn test_a_function_given_a_lambda_calls_it_by_name() {
        let dump =
            build_run_and_read_rc_ir(FOLD_SOURCE, "max", FOLD_OUTPUT, "a fold given a lambda");
        assert_calls_by_name(&dump, "Std::Iterator::fold");

        let dump =
            build_run_and_read_rc_ir(FOLD_SOURCE, "basic", FOLD_OUTPUT, "a fold given a lambda");
        assert!(
            calls_through_a_closure(&dump, "Std::Iterator::fold"),
            "at `basic` the fold should call its lambda through a closure"
        );
    }

    /// `relay` never calls the function it is given. The function reaches `terminal`, which calls
    /// it, only by being captured into the lambda `relay` hands to `fold`. Both are recursive, so
    /// both stay functions of their own.
    const RELAYED_SOURCE: &str = r#"
    module Main;

    terminal : (I64 -> I64) -> I64 -> I64;
    terminal = |op, n| (
        if n == 0 { 0 };
        op(n) + terminal(op, n - 1)
    );

    relay : (I64 -> I64) -> I64 -> I64;
    relay = |op, n| (
        if n == 0 { 0 };
        Iterator::range(0, n).fold(0, |i, acc| acc + terminal(op, i)) + relay(op, n - 1)
    );

    main : IO () = (
        println(relay(|x| x * 2, 4).to_string)
    );
    "#;

    /// What `RELAYED_SOURCE` prints: `relay(op, n)` sums `terminal(op, i)` over `0..n` and
    /// recurses on `n - 1`, with `op = |x| x * 2` and `n = 4`.
    const RELAYED_OUTPUT: &str = "30";

    /// A function reached only through the capture of a lambda is called by name where it is
    /// called: the set of the captured value is carried through the capture struct.
    #[test]
    fn test_a_function_reached_through_a_capture_is_called_by_name() {
        let dump = build_run_and_read_rc_ir(
            RELAYED_SOURCE,
            "max",
            RELAYED_OUTPUT,
            "a function relayed through a capture",
        );
        assert_calls_by_name(&dump, "Main::terminal");
        assert_calls_by_name(&dump, "Main::relay");
    }

    /// `RELAYED_SOURCE` with two functions captured into the one lambda `relay` hands to `fold`,
    /// each reaching a function of its own.
    const TWO_RELAYED_SOURCE: &str = r#"
    module Main;

    terminal_a : (I64 -> I64) -> I64 -> I64;
    terminal_a = |op, n| (
        if n == 0 { 0 };
        op(n) + terminal_a(op, n - 1)
    );

    terminal_b : (I64 -> I64) -> I64 -> I64;
    terminal_b = |op, n| (
        if n == 0 { 0 };
        op(n) * 2 + terminal_b(op, n - 1)
    );

    relay : (I64 -> I64) -> (I64 -> I64) -> I64 -> I64;
    relay = |f, g, n| (
        if n == 0 { 0 };
        Iterator::range(0, n).fold(0, |i, acc| acc + terminal_a(f, i) + terminal_b(g, i))
            + relay(f, g, n - 1)
    );

    main : IO () = (
        println(relay(|x| x * 2, |x| x * 3, 4).to_string)
    );
    "#;

    /// What `TWO_RELAYED_SOURCE` prints: `relay(f, g, n)` sums `terminal_a(f, i)` and
    /// `terminal_b(g, i)` over `0..n` and recurses on `n - 1`, with `f = |x| x * 2`,
    /// `g = |x| x * 3` and `n = 4`.
    const TWO_RELAYED_OUTPUT: &str = "120";

    /// Two functions captured into one lambda keep a set each: both are called by name where they
    /// are called.
    #[test]
    fn test_two_functions_captured_into_one_lambda_are_both_called_by_name() {
        let dump = build_run_and_read_rc_ir(
            TWO_RELAYED_SOURCE,
            "max",
            TWO_RELAYED_OUTPUT,
            "two functions relayed through one capture",
        );
        assert_calls_by_name(&dump, "Main::terminal_a");
        assert_calls_by_name(&dump, "Main::terminal_b");
        assert_calls_by_name(&dump, "Main::relay");
    }

    /// A loop whose body reads the array it captures.
    const LOOP_SOURCE: &str = r#"
    module Main;

    first_at_least : I64 -> Array I64 -> I64;
    first_at_least = |bound, values| (
        let n = values.@size;
        loop(0, |i| (
            if i == n { break $ -1 };
            if values.@(i) >= bound { break $ i };
            continue $ i + 1
        ))
    );

    main : IO ();
    main = (
        let values = [0, 1, 4, 9, 3, 12, 10, 10, 12, 3];
        println $ values.first_at_least(5).to_string
    );
    "#;

    /// What `LOOP_SOURCE` prints: the first element at or above 5 sits at index 3.
    const LOOP_OUTPUT: &str = "3";

    /// The body of a loop goes into the copy of `Std::loop` made for it: the copy reads the array
    /// through the index primitive itself.
    #[test]
    fn test_a_loop_body_goes_into_the_loop_made_for_it() {
        let dump =
            build_run_and_read_rc_ir(LOOP_SOURCE, "max", LOOP_OUTPUT, "a loop reading an array");
        assert_calls_by_name(&dump, "Std::loop");
        assert!(
            rc_ir_function_bodies(&dump, "Std::loop")
                .iter()
                .any(|body| body.contains("array_get(")),
            "a copy of `Std::loop` should read the array itself"
        );
    }

    /// `sum_up` and `sum_down` are given one lambda, and `twice` another. All three are recursive,
    /// so the inliner leaves them functions of their own.
    const SHARED_LAMBDA_SOURCE: &str = r#"
    module Main;

    twice : (I64 -> I64) -> I64 -> I64;
    twice = |op, n| (
        if n <= 0 { 0 };
        op(n) + op(n - 1) + twice(op, n - 2)
    );

    sum_up : (I64 -> I64) -> I64 -> I64;
    sum_up = |op, n| (
        if n <= 0 { 0 };
        op(n) + sum_up(op, n - 1)
    );

    sum_down : (I64 -> I64) -> I64 -> I64;
    sum_down = |op, n| (
        if n <= 0 { 0 };
        op(0 - n) + sum_down(op, n - 1)
    );

    main : IO ();
    main = (
        let k = 3;
        let shared = |x| x * k;
        let called_twice = |x| x * k - x;
        println $ (twice(called_twice, 8) + sum_up(shared, 4) + sum_down(shared, 4)).to_string
    );
    "#;

    /// What `SHARED_LAMBDA_SOURCE` prints: `twice` over `|x| x * 3 - x` at 8, and `sum_up` and
    /// `sum_down` over `|x| x * 3` at 4, which cancel.
    const SHARED_LAMBDA_OUTPUT: &str = "72";

    /// A function given a lambda takes that lambda's capture struct, so two functions given one
    /// lambda take one capture struct, and a function given another lambda takes another.
    #[test]
    fn test_two_functions_given_one_lambda_take_one_capture_struct() {
        let dump = build_run_and_read_rc_ir(
            SHARED_LAMBDA_SOURCE,
            "max",
            SHARED_LAMBDA_OUTPUT,
            "functions sharing a lambda",
        );
        let capture_structs_of = |name_part: &str| -> Vec<String> {
            assert_calls_by_name(&dump, name_part);
            let mut structs: Vec<String> = rc_ir_function_bodies(&dump, name_part)
                .iter()
                .flat_map(|body| capture_struct_parameters(header(body)))
                .map(|s| s.to_string())
                .collect();
            structs.sort();
            structs.dedup();
            assert_eq!(
                structs.len(),
                1,
                "the functions named with `{}` should take the capture struct of the one lambda \
                 they are given, and they take {:?}",
                name_part,
                structs
            );
            structs
        };
        let sum_up = capture_structs_of("Main::sum_up");
        assert_eq!(sum_up, capture_structs_of("Main::sum_down"));
        assert_ne!(sum_up, capture_structs_of("Main::twice"));
    }

    /// The loop in `main` hands the array it captures to `scaled_sum`, which reads it without
    /// keeping any of it.
    const LENDING_LOOP_SOURCE: &str = r#"
    module Main;

    scaled_sum : I64 -> Array I64 -> I64;
    scaled_sum = |key, table| (
        let n = table.@size;
        loop((0, 0), |(j, acc)| (
            if j == n { break $ acc };
            let x = table.@(j);
            let acc = if x % 7 == key % 7 { acc + x * 3 - j } else { acc + 1 };
            continue $ (j + 1, acc)
        ))
    );

    // The array is built at run time, so that the loop's lambda captures it: an array literal of
    // numbers is a constant, which the optimizer puts into the lambda instead.
    main : IO ();
    main = (
        let values = Array::from_map(10, |i| i * i % 13);
        let total = loop((0, 0), |(i, acc)| (
            if i == 4 { break $ acc };
            continue $ (i + 1, acc + scaled_sum(i, values))
        ));
        println $ total.to_string
    );
    "#;

    /// What `LENDING_LOOP_SOURCE` prints: `scaled_sum` over the array for the keys 0 to 3.
    const LENDING_LOOP_OUTPUT: &str = "111";

    /// A loop body that hands a value it captured to a call goes into the copy of `Std::loop` made
    /// for it like any other. The array then reaches the call as an argument of that copy, and the
    /// call goes to the callee's borrow version, so the copy retains no reference to the array.
    #[test]
    fn test_a_loop_handing_its_captured_array_to_a_call_counts_no_reference() {
        let dump = build_run_and_read_rc_ir(
            LENDING_LOOP_SOURCE,
            "max",
            LENDING_LOOP_OUTPUT,
            "a loop lending the array it captures",
        );
        assert_calls_by_name(&dump, "Std::loop");
        let lending: Vec<String> = rc_ir_function_bodies(&dump, "Std::loop")
            .into_iter()
            .filter(|body| !header(body).split('(').next().unwrap().contains("#borrow"))
            .filter(|body| {
                body.lines()
                    .any(|l| rc_ir_callee_of(l).is_some_and(|c| c.contains("#borrow")))
            })
            .collect();
        assert!(
            !lending.is_empty(),
            "the copy of `Std::loop` made for the loop in `main` should hand the array to a \
             borrow version"
        );
        for body in lending {
            assert!(
                !body.contains("retain"),
                "the copy of `Std::loop` lending the array should count no reference for it:\n{}",
                body
            );
        }
    }

    /// Two `map`s over an array put one iterator in a field of another, both of the same type
    /// constructor, and each carries the function it was given in a field of its own.
    const NESTED_ITERATORS_SOURCE: &str = r#"
    module Main;

    main : IO () = (
        let arr = Array::from_map(10, |i| i);
        println(arr.to_iter.map(|x| x * 3).map(|x| x + 1).fold(0, |x, acc| acc + x).to_string)
    );
    "#;

    /// What `NESTED_ITERATORS_SOURCE` prints: the sum of `3 * i + 1` over `0..9`.
    const NESTED_ITERATORS_OUTPUT: &str = "145";

    /// The functions an iterator carries in its fields reach the fold through the iterator's type:
    /// the fold takes the capture struct of its own operation, and an iterator whose type is a copy
    /// of `MapIterator` made for each of the two functions the chain carries. It calls each by name.
    #[test]
    fn test_a_fold_takes_the_functions_a_chain_of_iterators_carries_through_its_types() {
        let dump = build_run_and_read_rc_ir(
            NESTED_ITERATORS_SOURCE,
            "max",
            NESTED_ITERATORS_OUTPUT,
            "a fold over two maps",
        );
        assert_calls_by_name(&dump, "Std::Iterator::fold");
        let map_iterator_copy = format!("Std::Iterator::MapIterator{}", LS_COPY_SUFFIX);
        let takes_all_three = rc_ir_function_bodies(&dump, "Std::Iterator::fold")
            .iter()
            .any(|body| {
                let mut copies = parameter_tokens(header(body))
                    .filter(|token| token.starts_with(&map_iterator_copy))
                    .collect::<Vec<_>>();
                copies.sort();
                copies.dedup();
                capture_struct_parameters(header(body)).len() == 1 && copies.len() == 2
            });
        assert!(
            takes_all_three,
            "a fold should take the capture struct of its own operation and an iterator of two \
             copies of `MapIterator`, one for each function the chain carries:\n{}",
            rc_ir_function_bodies(&dump, "Std::Iterator::fold")
                .iter()
                .map(|body| header(body))
                .collect::<Vec<_>>()
                .join("\n")
        );
    }

    /// The functions the chain of iterators carries have their bodies put where the fold calls
    /// them: none stands as a function of its own. Each is lifted out of `main`, takes an `I64` and
    /// returns one, which distinguishes it from the IO plumbing also lifted out of `main`.
    #[test]
    fn test_the_functions_a_chain_of_iterators_carries_leave_no_function_of_their_own() {
        let dump = build_run_and_read_rc_ir(
            NESTED_ITERATORS_SOURCE,
            "max",
            NESTED_ITERATORS_OUTPUT,
            "a fold over two maps",
        );
        let standing: Vec<&str> = dump
            .lines()
            .filter(|line| line.starts_with("fn Main::main") && line.contains(LS_LAM_SUFFIX))
            .filter(|line| {
                line.ends_with(") -> Std::I64:") && parameters(line).contains("Std::I64")
            })
            .collect();
        assert!(
            standing.is_empty(),
            "no function the chain carries should stand as a function of its own, and these do:\n{}",
            standing.join("\n")
        );
    }

    /// A program whose `k` captures `g`, which `mk` builds from one of two functions, each read from
    /// an array of its own of `array_of_too_many_lambdas`. The sets of the two are different sets
    /// represented by closures, so the set of `g` holds `mk`'s lambda twice, two elements that
    /// differ only in sets of closures.
    fn two_alike_elements_source() -> String {
        format!(
            r#"
    module Main;

    mk : I64 -> (I64 -> I64) -> I64 -> I64;
    mk = |d, f| if d > 100 {{ mk(d - 1, f) }} else {{ |x| f(x) + 1 }};

    k : I64 -> (I64 -> I64) -> (I64 -> I64) -> I64 -> I64;
    k = |d, f1, f2| if d > 100 {{ k(d - 1, f1, f2) }} else {{
        let g = if d == 1 {{ mk(d, f1) }} else {{ mk(d, f2) }};
        |y| g(y) + d
    }};

    main : IO () = (
        let n = *get_args.map(|args| args.@size);
        let h1 = {}.@(n);
        let h2 = {}.@(n + 1);
        println((k(n, h1, h2)(4) + k(n + 1, h1, h2)(4)).to_string)
    );
    "#,
            array_of_too_many_lambdas("+"),
            array_of_too_many_lambdas("-")
        )
    }

    /// What `two_alike_elements_source` prints: `k(1, ..)` adds 2 to `h1(4)`, which is 5, and
    /// `k(2, ..)` adds 3 to `h2(4)`, which is 2.
    const TWO_ALIKE_ELEMENTS_OUTPUT: &str = "12";

    /// A set whose elements make one class is represented by that class's capture struct, however
    /// many elements it holds: `mk` builds `g` as a capture struct, and no closure.
    #[test]
    fn test_a_set_of_two_elements_of_one_class_is_a_capture_struct() {
        let dump = build_run_and_read_rc_ir(
            &two_alike_elements_source(),
            "max",
            TWO_ALIKE_ELEMENTS_OUTPUT,
            "a lambda capturing a function built from one of two alike functions",
        );
        for body in function_bodies_named(&dump, "Main::mk") {
            assert!(
                !body.contains("= closure "),
                "`mk` should build `g` as a capture struct, and this function builds a closure:\n{}",
                body
            );
        }
    }

    /// Whether the function header `header` takes a value of a union, which is how the rewrite
    /// represents a set of two or more classes.
    fn takes_a_union(header: &str) -> bool {
        parameters(header).contains(LS_UNION_PREFIX)
    }

    /// `apply` given one of two lambdas an `if` chooses. Recursive, so it stays a function of its
    /// own.
    const CHOSEN_SOURCE: &str = r#"
    module Main;

    apply : (I64 -> I64) -> I64 -> I64;
    apply = |f, x| if x > 100 { 0 } else { f(x) + apply(f, x + 1) };

    main : IO () = (
        let n = Array::fill(1, 10).@(0);
        let f = if n > 5 { |x| x + n } else { |x| x * 2 };
        println(apply(f, 3).to_string)
    );
    "#;

    /// What `CHOSEN_SOURCE` prints: the sum of `x + 10` over `3..100`.
    const CHOSEN_OUTPUT: &str = "6027";

    /// A set of two lambdas is represented by a union, and a function given a value of it calls
    /// each lambda by name: `apply` takes the union and builds and calls no closure. At `basic`,
    /// where the rewrite does not run, `apply` calls its function through a closure.
    #[test]
    fn test_a_function_given_one_of_two_lambdas_calls_each_by_name() {
        let dump = build_run_and_read_rc_ir(
            CHOSEN_SOURCE,
            "max",
            CHOSEN_OUTPUT,
            "a function given one of two lambdas",
        );
        assert_calls_by_name(&dump, "Main::apply");
        assert!(
            rc_ir_function_bodies(&dump, "Main::apply")
                .iter()
                .any(|body| takes_a_union(header(body))),
            "`apply` should take the union the two lambdas' set is represented by"
        );

        let dump = build_run_and_read_rc_ir(
            CHOSEN_SOURCE,
            "basic",
            CHOSEN_OUTPUT,
            "a function given one of two lambdas",
        );
        assert!(
            calls_through_a_closure(&dump, "Main::apply"),
            "at `basic` `apply` should call its function through a closure"
        );
    }

    /// `apply`, referring to a global array of functions, given a different lambda at each of two
    /// calls. Recursive, so it stays a function of its own.
    const APPLY_READING_A_GLOBAL_SOURCE: &str = r#"
    module Main;

    tbl : Array (I64 -> I64) = [|x| x + 1];

    apply : (I64 -> I64) -> I64 -> I64;
    apply = |f, x| if x > 100 { 0 } else { f(x) + (tbl.@(0))(x) + apply(f, x + 1) };

    main : IO () = (
        let n = Array::fill(1, 10).@(0);
        println((apply(|x| x + n, 3) + apply(|x| x * 2, 3)).to_string)
    );
    "#;

    /// What `APPLY_READING_A_GLOBAL_SOURCE` prints: the sums of `(x + 10) + (x + 1)` and of
    /// `x * 2 + (x + 1)` over `3..100`.
    const APPLY_READING_A_GLOBAL_OUTPUT: &str = "26411";

    /// A function that refers to a global value whose body is not a lambda, where no such global
    /// lies in the function's own component, is copied for each lambda set it is given: each copy of
    /// `apply` takes the set of the one lambda its call gives it, and none takes a union of the two.
    #[test]
    fn test_a_function_referring_to_a_pinned_global_is_copied_for_each_lambda_it_is_given() {
        let dump = build_run_and_read_rc_ir(
            APPLY_READING_A_GLOBAL_SOURCE,
            "max",
            APPLY_READING_A_GLOBAL_OUTPUT,
            "a function referring to a global array, given two lambdas",
        );
        assert_calls_by_name(&dump, "Main::apply");
        for body in rc_ir_function_bodies(&dump, "Main::apply") {
            assert!(
                !takes_a_union(header(&body)),
                "each copy of `apply` should take the set of the one lambda its call gives it, \
                 and this one takes a union:\n{}",
                body
            );
        }
    }

    /// An array holding two lambdas and a global function, folded over by applying each to the
    /// accumulator. The array's element set holds three classes.
    const ARRAY_SOURCE: &str = r#"
    module Main;

    triple : I64 -> I64;
    triple = |x| x * 3;

    main : IO () = (
        let n = Array::fill(1, 5).@(0);
        let fs = [|x| x + n, triple, |x| x - 1];
        println(fs.to_iter.fold(1, |f, acc| f(acc)).to_string)
    );
    "#;

    /// What `ARRAY_SOURCE` prints: `((1 + 5) * 3) - 1`.
    const ARRAY_OUTPUT: &str = "17";

    /// The elements of an array of functions are values of a union, and the fold calls each by
    /// name, the global function among them: the fold's copy builds and calls no closure.
    #[test]
    fn test_a_fold_over_an_array_of_functions_calls_each_by_name() {
        let dump = build_run_and_read_rc_ir(
            ARRAY_SOURCE,
            "max",
            ARRAY_OUTPUT,
            "a fold over an array of functions",
        );
        assert_calls_by_name(&dump, "Std::Iterator::fold");
        assert!(
            dump.contains(LS_UNION_PREFIX),
            "the array's elements should be values of a union:\n{}",
            dump
        );
    }

    /// A lambda capturing one of two lambdas an `if` chooses, handed to `apply`.
    const CAPTURED_CHOICE_SOURCE: &str = r#"
    module Main;

    apply : (I64 -> I64) -> I64 -> I64;
    apply = |f, x| if x > 100 { 0 } else { f(x) + apply(f, x + 1) };

    main : IO () = (
        let n = Array::fill(1, 10).@(0);
        let f = if n > 5 { |x| x + n } else { |x| x * 2 };
        let g = |x| f(x) + 1;
        println(apply(g, 3).to_string)
    );
    "#;

    /// What `CAPTURED_CHOICE_SOURCE` prints: the sum of `x + 11` over `3..100`.
    const CAPTURED_CHOICE_OUTPUT: &str = "6125";

    /// A union held in a capture struct is called by name: the copy of `apply` given `g` matches
    /// the union `g` captures and calls the chosen lambda, and builds and calls no closure.
    #[test]
    fn test_a_lambda_capturing_one_of_two_lambdas_calls_it_by_name() {
        let dump = build_run_and_read_rc_ir(
            CAPTURED_CHOICE_SOURCE,
            "max",
            CAPTURED_CHOICE_OUTPUT,
            "a lambda capturing one of two lambdas",
        );
        assert_calls_by_name(&dump, "Main::apply");
    }

    /// A program folding over an array of `count` lambdas, the `i`-th adding `i` times a number the
    /// compiler does not know, so that every lambda is a class of its own.
    fn many_lambdas_source(count: usize) -> String {
        let lambdas: Vec<String> = (0..count).map(|i| format!("|x| x + n * {}", i)).collect();
        format!(
            r#"
    module Main;

    main : IO () = (
        let n = Array::fill(1, 1).@(0);
        let fs = [{}];
        println(fs.to_iter.fold(0, |f, acc| f(acc)).to_string)
    );
    "#,
            lambdas.join(", ")
        )
    }

    /// What `many_lambdas_source(count)` prints: the sum of `0..count`.
    fn many_lambdas_output(count: usize) -> String {
        (count * (count - 1) / 2).to_string()
    }

    /// A set of as many classes as `MAX_LS_UNION_VARIANTS` is represented by a union, and one of a
    /// class more by closures: the fold over that many lambdas takes their union, and the fold over
    /// one lambda more calls them through closures, and both print the expected sum.
    #[test]
    fn test_a_set_of_too_many_classes_is_a_set_of_closures() {
        let source = many_lambdas_source(MAX_LS_UNION_VARIANTS);
        let dump = build_run_and_read_rc_ir(
            &source,
            "max",
            &many_lambdas_output(MAX_LS_UNION_VARIANTS),
            "a fold over `MAX_LS_UNION_VARIANTS` lambdas",
        );
        assert_calls_by_name(&dump, "Std::Iterator::fold");
        assert!(
            dump.contains(LS_UNION_PREFIX),
            "the lambdas should be values of a union"
        );

        let source = many_lambdas_source(MAX_LS_UNION_VARIANTS + 1);
        let dump = build_run_and_read_rc_ir(
            &source,
            "max",
            &many_lambdas_output(MAX_LS_UNION_VARIANTS + 1),
            "a fold over one lambda more than `MAX_LS_UNION_VARIANTS`",
        );
        assert!(
            !dump.contains(LS_UNION_PREFIX),
            "no union should stand for the lambdas"
        );
        assert!(
            calls_through_a_closure(&dump, "Std::Iterator::fold"),
            "the fold should call the lambdas through closures"
        );
    }

    /// Only the first set of a chain past the bound on the capture structs a value lays out in place
    /// (`MAX_LS_IN_PLACE_CAPTURE_STRUCTS`) is represented by closures: the sets outside it hold that
    /// set's closures, which lay out none, so they are counted afresh and are unions again. The last
    /// function of `chain_source` is applied where it is built, which leaves no set of its own, so the
    /// dump names a union of `Main` for every other function but those the count puts past the bound.
    #[test]
    fn test_only_the_first_set_of_a_chain_past_the_bound_is_closures() {
        let dump = build_run_and_read_rc_ir(
            &chain_source(),
            "max",
            CHAIN_OUTPUT,
            "a chain of unions each capturing the one before",
        );
        let main_union = format!("Main::{}", LS_UNION_PREFIX);
        let unions: Set<&str> = dump
            .match_indices(&main_union)
            .map(|(at, _)| {
                let rest = &dump[at..];
                &rest[..rest
                    .find(|c: char| c.is_whitespace() || c == ')' || c == ',')
                    .unwrap()]
            })
            .collect();
        // A value of the `j`-th function since the last one past the bound, `f0` counting as the
        // 0th, lays out `2^(j + 2) - 2` capture structs in place.
        let mut past_the_bound = 0;
        let mut since = 0;
        for _ in 0..CHAIN_DEPTH {
            if (1u64 << (since + 2)) - 2 > MAX_LS_IN_PLACE_CAPTURE_STRUCTS {
                past_the_bound += 1;
                since = 0;
            } else {
                since += 1;
            }
        }
        assert!(
            past_the_bound > 0,
            "the chain should reach past the bound for the test to judge anything"
        );
        assert_eq!(
            unions.len(),
            CHAIN_DEPTH - past_the_bound,
            "every function of the chain but the last and those past the bound should have a union: \
             {:?}",
            unions
        );
    }

    /// A program declaring `decls` and building `f`, a lambda capturing `words` integers the
    /// compiler does not know, each laid out in one word, and printing `use_f`, an `I64` reading
    /// `f`. `f` adds `wide_capture_sum(words)` to its argument.
    fn wide_capture_source(words: usize, decls: &str, use_f: &str) -> String {
        let lets: Vec<String> = (1..=words)
            .map(|i| format!("let c{} = n + {};", i, i))
            .collect();
        let addends: Vec<String> = (1..=words).map(|i| format!(" + c{}", i)).collect();
        format!(
            r#"
    module Main;

    {}

    main : IO () = (
        let n = Array::fill(1, 1).@(0);
        {}
        let f = |x| x{};
        println(({}).to_string)
    );
    "#,
            decls,
            lets.join("\n        "),
            addends.join(""),
            use_f
        )
    }

    /// What `f` of `wide_capture_source(words, ..)` adds to its argument.
    fn wide_capture_sum(words: usize) -> usize {
        (1..=words).map(|i| 1 + i).sum()
    }

    /// Folds over an array filled with `f` of `wide_capture_source`.
    const FOLD_OVER_AN_ARRAY_OF_F: &str = "Array::fill(3, f).to_iter.fold(0, |g, acc| g(acc))";

    /// Whether the capture struct of `Main` with `field_count` fields that the dump builds is boxed.
    /// The dump writes after a value the boxed objects it holds and where they come from: a boxed value
    /// as one source (`[fresh]`), an unboxed value as `[unboxed]` when it holds no boxed object and
    /// as its fields' sources in braces otherwise. Panics when the dump builds no such capture
    /// struct, or builds one both boxed and unboxed.
    fn main_capture_struct_is_boxed(dump: &str, field_count: usize) -> bool {
        let boxed: Set<bool> = dump
            .lines()
            .filter(|line| line.contains(&format!(": Main::{}@", LS_CAP_PREFIX)))
            .filter(|line| !line.contains("(as #decapture"))
            .filter_map(|line| {
                let (head, args) = line.split_once("= struct_make(")?;
                let args = args.trim_end().strip_suffix(')')?;
                let arity = if args.is_empty() {
                    0
                } else {
                    args.split(", ").count()
                };
                let (_, provenance) = head.trim_end().rsplit_once('[')?;
                (arity == field_count)
                    .then(|| provenance != "unboxed]" && !provenance.starts_with('{'))
            })
            .collect();
        assert_eq!(
            boxed.len(),
            1,
            "the dump should build the capture struct of {} values, boxed or unboxed:\n{}",
            field_count,
            dump
        );
        boxed.contains(&true)
    }

    /// A set whose values an array stores is represented by an unboxed capture struct while its
    /// values take `MAX_LS_STORED_VALUE_WORDS` words, and by a boxed one once they take more: the
    /// fold over an array filled with a lambda calls it by name either way.
    #[test]
    fn test_a_set_an_array_stores_is_a_boxed_capture_struct_past_the_stored_words() {
        let words = MAX_LS_STORED_VALUE_WORDS;
        let dump = build_run_and_read_rc_ir(
            &wide_capture_source(words, "", FOLD_OVER_AN_ARRAY_OF_F),
            "max",
            &(3 * wide_capture_sum(words)).to_string(),
            "a fold over an array of a lambda capturing `MAX_LS_STORED_VALUE_WORDS` words",
        );
        assert_calls_by_name(&dump, "Std::Iterator::fold");
        assert!(
            !main_capture_struct_is_boxed(&dump, words),
            "the capture struct should be unboxed"
        );

        let words = MAX_LS_STORED_VALUE_WORDS + 1;
        let dump = build_run_and_read_rc_ir(
            &wide_capture_source(words, "", FOLD_OVER_AN_ARRAY_OF_F),
            "max",
            &(3 * wide_capture_sum(words)).to_string(),
            "a fold over an array of a lambda capturing a word more than \
             `MAX_LS_STORED_VALUE_WORDS`",
        );
        assert_calls_by_name(&dump, "Std::Iterator::fold");
        assert!(
            main_capture_struct_is_boxed(&dump, words),
            "the capture struct should be boxed"
        );
    }

    /// A list of functions, each node a boxed union holding one, and `apply_in_turn`, which applies them in
    /// turn.
    const LIST_OF_FUNCTIONS_DECLS: &str = r#"
    type Fns = box union { nil : (), cons : (I64 -> I64, Fns) };

    apply_in_turn : Fns -> I64 -> I64;
    apply_in_turn = |fs, acc| match fs {
        nil(_) => acc,
        cons((g, rest)) => apply_in_turn(rest, g(acc))
    };"#;

    /// Applies a list of two `f` of `wide_capture_source`.
    const SUM_OVER_A_LIST_OF_F: &str =
        "apply_in_turn(Fns::cons((f, Fns::cons((f, Fns::nil())))), 0)";

    /// A set whose values a boxed union stores is represented by an unboxed capture struct while its
    /// values take `MAX_LS_STORED_VALUE_WORDS` words, and by a boxed one once they take more:
    /// `apply_in_turn` calls the lambda the list holds by name either way.
    #[test]
    fn test_a_set_a_boxed_union_stores_is_a_boxed_capture_struct_past_the_stored_words() {
        let words = MAX_LS_STORED_VALUE_WORDS;
        let dump = build_run_and_read_rc_ir(
            &wide_capture_source(words, LIST_OF_FUNCTIONS_DECLS, SUM_OVER_A_LIST_OF_F),
            "max",
            &(2 * wide_capture_sum(words)).to_string(),
            "a list of a lambda capturing `MAX_LS_STORED_VALUE_WORDS` words",
        );
        assert_calls_by_name(&dump, "Main::apply_in_turn");
        assert!(
            !main_capture_struct_is_boxed(&dump, words),
            "the capture struct should be unboxed"
        );

        let words = MAX_LS_STORED_VALUE_WORDS + 1;
        let dump = build_run_and_read_rc_ir(
            &wide_capture_source(words, LIST_OF_FUNCTIONS_DECLS, SUM_OVER_A_LIST_OF_F),
            "max",
            &(2 * wide_capture_sum(words)).to_string(),
            "a list of a lambda capturing a word more than `MAX_LS_STORED_VALUE_WORDS`",
        );
        assert_calls_by_name(&dump, "Main::apply_in_turn");
        assert!(
            main_capture_struct_is_boxed(&dump, words),
            "the capture struct should be boxed"
        );
    }

    /// A lambda capturing two arrays, stored in an array, read out of it and called once, and
    /// updating in place the array it captured, which it asserts unique.
    const UNIQUE_CAPTURE_SOURCE: &str = r#"
    module Main;

    main : IO () = (
        let n = *get_args.map(|args| args.@size);
        let a0 = Array::fill(n, 0);
        let a1 = Array::fill(n + 1, 1);
        let f = |i| a0.assert_unique_array(|_| "the array the lambda captured").set(0, i).@(0) + a1.@size;
        let fs = Array::fill(1, f);
        println((fs.@(0))(7).to_string)
    );
    "#;

    /// A value read out of a boxed capture struct at the struct's last use holds a reference of its
    /// own, so once the struct is freed the value is unique again: the lambda called once updates
    /// the array it captured in place.
    #[test]
    fn test_a_value_read_out_of_a_boxed_capture_struct_at_its_last_use_is_unique() {
        let dump = build_run_and_read_rc_ir(
            UNIQUE_CAPTURE_SOURCE,
            "max",
            "9",
            "a lambda stored in an array updating the array it captured",
        );
        assert!(
            main_capture_struct_is_boxed(&dump, 2),
            "the capture struct should be boxed"
        );
    }

    /// A set whose values nothing stores in an array or a boxed object is represented by an unboxed
    /// capture struct however many words its values take: a loop calling a lambda capturing more
    /// than `MAX_LS_STORED_VALUE_WORDS` words calls it by name, through the unboxed capture struct
    /// of the lambda it is given.
    #[test]
    fn test_a_set_stored_in_no_heap_object_is_a_capture_struct_however_wide() {
        let words = MAX_LS_STORED_VALUE_WORDS + 1;
        let dump = build_run_and_read_rc_ir(
            &wide_capture_source(words, "", "range(0, 3).fold(0, |_, acc| f(acc))"),
            "max",
            &(3 * wide_capture_sum(words)).to_string(),
            "a loop calling a lambda capturing more than `MAX_LS_STORED_VALUE_WORDS` words",
        );
        assert_calls_by_name(&dump, "Std::Iterator::fold");
        assert!(
            !main_capture_struct_is_boxed(&dump, words),
            "the capture struct should be unboxed"
        );
    }

    /// A set the capture struct of a lambda of a set of closures holds in place is held in the
    /// closure's object, so it is boxed past `MAX_LS_STORED_VALUE_WORDS` words as one an array
    /// stores is: the lambda `wrap` returns captures a value of its own set, which is so represented
    /// by closures, and captures `f`, whose capture struct is boxed.
    #[test]
    fn test_a_set_a_closure_holds_in_place_is_a_boxed_capture_struct_past_the_stored_words() {
        let source = r#"
    module Main;

    // Wraps `base` `n` times in a lambda applying `f` first.
    wrap : I64 -> (I64 -> I64) -> (I64 -> I64) -> (I64 -> I64);
    wrap = |n, f, base| if n == 0 { base } else { let g = wrap(n - 1, f, base); |x| g(f(x)) };

    main : IO () = (
        let n = Array::fill(1, 1).@(0);
        let c1 = n + 1;
        let c2 = n + 2;
        let c3 = n + 3;
        let c4 = n + 4;
        let c5 = n + 5;
        let f = |x| x + c1 + c2 + c3 + c4 + c5;
        let gs = Array::from_map(3, |i| wrap(1, f, |x| x + i));
        println(gs.to_iter.fold(0, |g, acc| g(acc)).to_string)
    );
    "#;
        let dump = build_run_and_read_rc_ir(
            source,
            "max",
            "63",
            "an array of closures each capturing a lambda that captures five words",
        );
        assert!(
            main_capture_struct_is_boxed(&dump, 5),
            "the capture struct of `f` should be boxed"
        );
    }

    /// A set of three lambdas whose values an array stores, a union taking more than
    /// `MAX_LS_STORED_VALUE_WORDS` words, boxes the capture structs of its classes that take more
    /// than a word, and leaves the one taking a word unboxed: the fold over the array calls each
    /// lambda by name.
    #[test]
    fn test_a_set_an_array_stores_boxes_the_classes_wider_than_a_word() {
        let source = r#"
    module Main;

    main : IO () = (
        let n = Array::fill(1, 1).@(0);
        let c1 = n + 1;
        let c2 = n + 2;
        let c3 = n + 3;
        let c4 = n + 4;
        let c5 = n + 5;
        let f = |x| x + c1 + c2 + c3 + c4 + c5;
        let g = |x| x + c1 + c2;
        let h = |x| x + c3;
        println([f, g, h].to_iter.fold(0, |k, acc| k(acc)).to_string)
    );
    "#;
        let dump = build_run_and_read_rc_ir(
            source,
            "max",
            "29",
            "a fold over an array of three lambdas capturing five, two and one words",
        );
        assert_calls_by_name(&dump, "Std::Iterator::fold");
        assert!(
            main_capture_struct_is_boxed(&dump, 5),
            "the capture struct of five words should be boxed"
        );
        assert!(
            main_capture_struct_is_boxed(&dump, 2),
            "the capture struct of two words should be boxed"
        );
        assert!(
            !main_capture_struct_is_boxed(&dump, 1),
            "the capture struct of one word should be unboxed"
        );
    }

    /// A lambda `h` that an array stores and that captures, beside two integers, a lambda `f` whose
    /// capture struct is boxed counts `f` as the word it takes boxed: `h` takes three words, within
    /// `MAX_LS_STORED_VALUE_WORDS`, and its capture struct stays unboxed.
    #[test]
    fn test_a_set_an_array_stores_counts_a_held_boxed_capture_struct_as_a_word() {
        let source = r#"
    module Main;

    main : IO () = (
        let n = Array::fill(1, 1).@(0);
        let c1 = n + 1;
        let c2 = n + 2;
        let c3 = n + 3;
        let c4 = n + 4;
        let c5 = n + 5;
        let f = |x| x + c1 + c2 + c3 + c4 + c5;
        let d1 = n + 6;
        let d2 = n + 7;
        let h = |x| f(x) + d1 + d2;
        let s = Array::fill(3, f).to_iter.fold(0, |k, acc| k(acc));
        let t = Array::fill(3, h).to_iter.fold(0, |k, acc| k(acc));
        println((s + t).to_string)
    );
    "#;
        let dump = build_run_and_read_rc_ir(
            source,
            "max",
            "165",
            "folds over an array of a lambda capturing five words and over an array of a lambda \
             capturing it and two words",
        );
        assert_calls_by_name(&dump, "Std::Iterator::fold");
        assert!(
            main_capture_struct_is_boxed(&dump, 5),
            "the capture struct of `f` should be boxed"
        );
        assert!(
            !main_capture_struct_is_boxed(&dump, 3),
            "the capture struct of `h` should be unboxed"
        );
    }

    /// The words a capture struct takes are those of the values it captures laid out in place, which
    /// an array spans three of: a lambda capturing two arrays takes six words, past
    /// `MAX_LS_STORED_VALUE_WORDS`, and an array of it holds it boxed.
    #[test]
    fn test_a_set_an_array_stores_counts_the_words_of_its_captured_values() {
        let source = r#"
    module Main;

    main : IO () = (
        let n = Array::fill(1, 1).@(0);
        let a = Array::fill(n, n + 1);
        let b = Array::fill(n, n + 2);
        let f = |x| x + a.@(0) + b.@(0);
        println(Array::fill(3, f).to_iter.fold(0, |k, acc| k(acc)).to_string)
    );
    "#;
        let dump = build_run_and_read_rc_ir(
            source,
            "max",
            "15",
            "a fold over an array of a lambda capturing two arrays",
        );
        assert_calls_by_name(&dump, "Std::Iterator::fold");
        assert!(
            main_capture_struct_is_boxed(&dump, 2),
            "the capture struct of the two arrays should be boxed"
        );
    }
}

/// Global values of function type that are not lambdas — a partial application, a struct of
/// functions, and two arrays an `if` in `main` chooses between — each referred to in several places
/// that give them lambdas of their own. Each body runs once, and the program computes them all.
#[test]
fn test_global_values_of_functions_referred_to_in_many_places_are_evaluated_once() {
    let source = r#"
    module Main;

    tbl : Array (I64 -> I64);
    tbl = (
        eval debug_eprintln("evaluating tbl");
        [|x| x + 1, |x| x * 2]
    );

    tbl2 : Array (I64 -> I64);
    tbl2 = (
        eval debug_eprintln("evaluating tbl2");
        [|x| x - 3]
    );

    adder : I64 -> I64 -> I64;
    adder = |a, b| if a > 1000 { adder(a - 1, b) } else { a + b };

    add5 : I64 -> I64;
    add5 = (
        eval debug_eprintln("evaluating add5");
        adder(5)
    );

    pick : (I64 -> I64) -> Bool -> (I64 -> I64);
    pick = |f, b| if b { f } else { tbl.@(0) };

    apply_all : Array (I64 -> I64) -> I64 -> I64;
    apply_all = |fs, x| fs.to_iter.fold(x, |f, acc| f(acc));

    type S = struct { f : I64 -> I64, g : I64 -> I64 };

    sv : S;
    sv = (
        eval debug_eprintln("evaluating sv");
        S { f : |x| x + 7, g : add5 }
    );

    main : IO ();
    main = (
        let n = *get_args.map(|args| args.@size);
        let p1 = pick(|x| x + 100 * n, true);
        let p2 = pick(|x| x * 3, false);
        let p3 = pick(|x| x * 5 - n, true);
        let chosen = if n == 1 { tbl } else { tbl2 };
        let chosen2 = if n == 2 { tbl } else { tbl2 };
        let r1 = apply_all(chosen, 10);
        let r2 = apply_all(chosen2, 10);
        let r3 = apply_all(tbl.push_back(add5), 1);
        let h = if n == 1 { add5 } else { |x| x * n };
        let s2 = sv.mod_f(|f| |x| f(x) + n);
        let s3 = sv.set_g(|x| x - n);
        let t = (s2.@f)(1) + (s2.@g)(1) + (s3.@f)(2) + (s3.@g)(2) + (sv.@g)(3);
        println([p1(1), p2(1), p3(1), r1, r2, r3, h(4), t].to_iter.map(to_string).join(" "))
    );
    "#;
    assert_prints_and_evaluates_each_once(
        source,
        "101 2 4 22 7 9 9 33",
        &["tbl", "tbl2", "add5", "sv"],
    );
}

/// A recursive function that pushes onto a global array of functions a lambda capturing its own
/// argument, called by `main` with two lambdas: the set of the array's elements holds the pushed
/// lambda, so the set of the argument it captures is one set too, holding both of `main`'s lambdas,
/// and the program computes every element.
#[test]
fn test_a_lambda_pushed_onto_a_global_array_capturing_an_argument_computes_it() {
    let source = r#"
    module Main;

    tbl : Array (I64 -> I64) = [|x| x + 1];

    ext : (I64 -> I64) -> I64 -> Array (I64 -> I64);
    ext = |g, k| if k <= 0 { tbl.push_back(|x| g(x) * 10) } else { ext(g, k - 1) };

    main : IO ();
    main = (
        let n = *get_args.map(|args| args.@size);
        let a = ext(|x| x + n, n);
        let m = 7 + n;
        let b = ext(|x| x * m, n);
        let c = a.append(b).to_iter.map(|f| f(3)).to_array;
        assert_eq(|_| "", c, [4, 40, 4, 240]);;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_mode());
}

/// A recursive function that pushes onto a global array of functions a closure that another
/// function builds from its own argument, called by `main` with two lambdas: the set of the
/// array's elements holds the other function's lambda, which captures the argument, so the set of
/// the argument is one set too, holding both of `main`'s lambdas, and the program computes every
/// element.
#[test]
fn test_a_closure_built_by_a_callee_pushed_onto_a_global_array_computes_it() {
    let source = r#"
    module Main;

    tbl : Array (I64 -> I64) = [|x| x + 1];

    times10 : (I64 -> I64) -> I64 -> (I64 -> I64);
    times10 = |g, d| if d > 0 { times10(g, d - 1) } else { |x| g(x) * 10 };

    ext : (I64 -> I64) -> I64 -> Array (I64 -> I64);
    ext = |g, k| if k <= 0 { tbl.push_back(times10(g, k)) } else { ext(g, k - 1) };

    main : IO ();
    main = (
        let n = *get_args.map(|args| args.@size);
        let a = ext(|x| x + n, n);
        let m = 7 + n;
        let b = ext(|x| x * m, n);
        let c = a.append(b).to_iter.map(|f| f(3)).to_array;
        assert_eq(|_| "", c, [4, 40, 4, 240]);;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_mode());
}

/// Verifies that when a function passes `both` the results of two functions that build different
/// lambdas, the function `both` returns calls the lambda of `ca` through its first capture and that
/// of `cb` through its second.
#[test]
fn test_two_functions_building_different_lambdas_passed_to_one_compute_each() {
    let source = r#"
    module Main;

    type P = box struct { run : I64 -> I64 };

    both : P -> P -> P;
    both = |a, b| if (a.@run)(0) < -1000000 { both(a, b) } else { P { run : |x| (a.@run)((b.@run)(x)) } };

    ca : I64 -> P;
    ca = |n| if n < -1000000 { ca(n) } else { P { run : |x| x + n } };

    cb : I64 -> P;
    cb = |n| if n < -1000000 { cb(n) } else { P { run : |x| x * n } };

    c : I64 -> P;
    c = |n| if n < -1000000 { c(n) } else { both(ca(n), cb(n)) };

    main : IO ();
    main = (
        let k = *get_args.map(|args| args.@size) + 2;
        assert_eq(|_| "", (c(k).@run)(10), 10 * k + k);;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_mode());
}

/// Arrays of functions moved by the builtins that carry an element without calling it — a swap, a
/// copy into a subarray, the element `act` reads, and the element `mod` punches out and plugs
/// back — and a function value returned by `unsafe_is_unique`: each keeps the lambdas it was given,
/// and the recursive function they reach calls them all.
#[test]
fn test_functions_carried_through_the_builtins_moving_them_compute_them() {
    let source = r#"
    module Main;

    type BF = box struct { f : I64 -> I64 };

    call_all : I64 -> Array (I64 -> I64) -> I64 -> I64;
    call_all = |d, fs, x| if d > 0 { call_all(d - 1, fs, x) } else { fs.to_iter.fold(0, |f, acc| acc * 100 + f(x)) };

    main : IO ();
    main = (
        let n = *get_args.map(|args| args.@size);
        let swapped = call_all(n, [|x| x + 10 + n, |x| x + 20].swap(0, 1), 1);
        let sub = call_all(n, [|x| x + 60 + n, |x| x + 70].get_sub(1, 2), 1);
        let acted = call_all(n, [|x| x + 11 + n].act(0, |q| Option::some(|x| q(x) + 1)).as_some, 1);
        let modded = call_all(n, [|x| x + 12 + n].mod(0, |q| |x| q(x) * 2), 1);
        let (_, bf) = (BF { f : |x| x + 9 + n }).unsafe_is_unique;
        let unique_checked = (bf.@f)(1);
        assert_eq(|_| "", [swapped, sub, acted, modded, unique_checked], [2111 + n, 71, 13 + n, 26 + 2 * n, 10 + n]);;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_mode());
}

/// A function given a value of a type whose field applies a type constructor argument: once to a
/// struct declaring a function field, once to a tuple type applied to a function type but not to
/// its second argument. The field's function reaches the function through a type the slots of the
/// field do not track, and the function calls it.
#[test]
fn test_a_function_in_a_type_given_as_a_type_constructor_argument_computes_it() {
    let source = r#"
    module Main;

    type [f : * -> *] W f = box struct { x : f I64 };
    type Fn a = box struct { g : a -> a, v : a };

    use_fn : I64 -> W Fn -> I64;
    use_fn = |d, w| if d > 0 { use_fn(d - 1, w) } else { (w.@x.@g)(w.@x.@v) };

    use_tup : I64 -> W (Tuple2 (I64 -> I64)) -> I64;
    use_tup = |d, w| if d > 0 { use_tup(d - 1, w) } else { (w.@x.@0)(w.@x.@1) };

    main : IO ();
    main = (
        let n = *get_args.map(|args| args.@size);
        assert_eq(|_| "through a struct", use_fn(n, W { x : Fn { g : |x| x * 10 + n, v : 4 } }), 40 + n);;
        assert_eq(|_| "through a tuple", use_tup(n, W { x : (|x| x * 100 + n, 5) }), 500 + n);;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_mode());
}

/// The field `t (Tuple2 (m I64))` applies one type constructor argument to `Tuple2` given the
/// other one applied. Here both are newtypes: `t` is a struct whose one field applies its argument,
/// and `m` is `IO`. The field is a tuple of an `IO` action and a number, and a recursive function
/// reading it runs the action.
#[test]
fn test_a_field_applying_a_newtype_to_a_tuple_type_holding_an_io_action_runs_it() {
    let source = r#"
    module Main;

    type [f : * -> *] NW f = unbox struct { x : f I64 };
    type [t : (* -> *) -> *, m : * -> *] Outer t m = box struct { o : t (Tuple2 (m I64)) };

    use_o : I64 -> Outer NW IO -> IO I64;
    use_o = |d, w| if d > 0 { use_o(d - 1, w) } else { let v = *w.@o.@x.@0; pure(v + w.@o.@x.@1) };

    main : IO ();
    main = (
        let n = *get_args.map(|args| args.@size);
        let sum = *use_o(n, Outer { o : NW { x : (pure(10 + n), 5) } });
        assert_eq(|_| "", sum, 15 + n);;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_mode());
}

/// The field `t (Tuple2 (m I64))` at an instance where `m` is a newtype of a number, so that the
/// field holds no function: a struct built by a literal and read by getters gives back the numbers
/// it was built with.
#[test]
fn test_a_field_applying_a_newtype_to_a_tuple_type_holding_a_newtype_of_a_number_reads_it() {
    let source = r#"
    module Main;

    type Nt a = unbox struct { v : a };
    type [f : * -> *] NW f = unbox struct { x : f I64 };
    type [t : (* -> *) -> *, m : * -> *] Outer t m = box struct { o : t (Tuple2 (m I64)) };

    main : IO ();
    main = (
        let n = *get_args.map(|args| args.@size);
        let w : Outer NW Nt = Outer { o : NW { x : (Nt { v : 10 + n }, 5) } };
        assert_eq(|_| "", w.@o.@x.@0.@v + w.@o.@x.@1, 15 + n);;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_mode());
}

/// Fields applying a type constructor argument `t` to a tuple type given all but its last
/// argument, with the newtype `m` at each place such a type can hold one: as a later argument
/// (`Later`), under a type constructor applied to all its arguments (`Under`), and applied to
/// itself (`Twice`). `Short` binds `t` to a newtype given one of its two arguments, and `Inner`
/// reaches the field through a newtype whose own field applies `t` the same way. Each field gives
/// back the numbers it was built with.
#[test]
fn test_fields_applying_a_newtype_to_types_holding_a_newtype_at_each_place_read_it() {
    let source = r#"
    module Main;

    type Nt a = unbox struct { v : a };
    type [f : * -> *] NW f = unbox struct { x : f I64 };
    type [f : * -> *] NW2 a f = unbox struct { x : f a };
    type [t : (* -> *) -> *, f : * -> *] NWW t f = unbox struct { y : t (Tuple2 (f I64)) };

    type [t : (* -> *) -> *, m : * -> *] Later t m = box struct { o : t (Tuple3 I64 (m I64)) };
    type [t : (* -> *) -> *, m : * -> *] Under t m = box struct { o : t (Tuple2 (Option (m I64))) };
    type [t : (* -> *) -> *, m : * -> *] Twice t m = box struct { o : t (Tuple2 (m (m I64))) };
    type [t : (* -> *) -> *, m : * -> *] Short t m = box struct { o : t (Tuple2 (m I64)) };
    type [p : ((* -> *) -> *) -> (* -> *) -> *, t : (* -> *) -> *, m : * -> *] Inner p t m = box struct { o : p t m };

    main : IO ();
    main = (
        let n = *get_args.map(|args| args.@size);
        let later : Later NW Nt = Later { o : NW { x : (1, Nt { v : 10 + n }, 5) } };
        let under : Under NW Nt = Under { o : NW { x : (Option::some(Nt { v : 20 + n }), 5) } };
        let twice : Twice NW Nt = Twice { o : NW { x : (Nt { v : Nt { v : 30 + n } }, 5) } };
        let short : Short (NW2 I64) Nt = Short { o : NW2 { x : (Nt { v : 40 + n }, 5) } };
        let inner : Inner NWW NW Nt = Inner { o : NWW { y : NW { x : (Nt { v : 50 + n }, 5) } } };
        assert_eq(|_| "", [
            later.@o.@x.@1.@v + later.@o.@x.@2,
            under.@o.@x.@0.as_some.@v + under.@o.@x.@1,
            twice.@o.@x.@0.@v.@v + twice.@o.@x.@1,
            short.@o.@x.@0.@v + short.@o.@x.@1,
            inner.@o.@y.@x.@0.@v + inner.@o.@y.@x.@1
        ], [15 + n, 25 + n, 35 + n, 45 + n, 55 + n]);;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_mode());
}

/// The field `t (Tuple2 (m I64))` of a union and of an array's elements, and that of a struct taken
/// apart by a pattern and rebuilt by `mod` and `set`: each gives the numbers it holds.
#[test]
fn test_a_field_applying_a_newtype_to_a_tuple_type_holding_a_newtype_is_read_each_way() {
    let source = r#"
    module Main;

    type Nt a = unbox struct { v : a };
    type [f : * -> *] NW f = unbox struct { x : f I64 };
    type [t : (* -> *) -> *, m : * -> *] Outer t m = box struct { o : t (Tuple2 (m I64)) };
    type [t : (* -> *) -> *, m : * -> *] OuterU t m = box union { o : t (Tuple2 (m I64)), e : () };
    type [t : (* -> *) -> *, m : * -> *] OuterA t m = box struct { o : Array (t (Tuple2 (m I64))) };

    main : IO ();
    main = (
        let n = *get_args.map(|args| args.@size);

        let u : OuterU NW Nt = OuterU::o(NW { x : (Nt { v : 10 + n }, 5) });
        let u = if n > 5 { OuterU::e() } else { u };
        let matched = match u { o(nw) => nw.@x.@0.@v + nw.@x.@1, e(_) => 0 };
        let taken = u.as_o.@x.@0.@v;

        let a : OuterA NW Nt = OuterA { o : [NW { x : (Nt { v : 20 + n }, 5) }] };
        let element = a.@o.@(0).@x.@0.@v + a.@o.@(0).@x.@1;

        let w : Outer NW Nt = Outer { o : NW { x : (Nt { v : 30 + n }, 5) } };
        let Outer { o : NW { x : (first, second) } } = w;
        let destructured = first.@v + second;

        let w = w.mod_o(|nw| nw.mod_x(|(a, b)| (a.mod_v(|v| v + 1), b + 1)));
        let w = w.set_o(NW { x : (Nt { v : w.@o.@x.@0.@v * 2 }, w.@o.@x.@1) });
        let rebuilt = w.@o.@x.@0.@v + w.@o.@x.@1;

        assert_eq(|_| "", [matched, taken, element, destructured, rebuilt], [15 + n, 10 + n, 25 + n, 35 + n, 68 + 2 * n]);;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_mode());
}

/// The field `t (Tuple2 (m I64))` beside fields declaring a function: `g`, a function type of its
/// own, and `p`, the argument `m` applied to one. With `m` a newtype of a function, a recursive
/// function calls the function the tuple holds and the one in `g`; with `m` a newtype of its
/// argument, it calls the function in `p`, and the one the field `t (Tuple2 (m (I64 -> I64)))` of
/// `Held` holds.
#[test]
fn test_functions_beside_and_inside_a_field_applying_a_newtype_to_a_tuple_type_compute_them() {
    let source = r#"
    module Main;

    type Nt a = unbox struct { v : a };
    type Fn a = unbox struct { f : a -> a };
    type [f : * -> *] NW f = unbox struct { x : f I64 };
    type [t : (* -> *) -> *, m : * -> *] Outer t m = box struct { g : I64 -> I64, o : t (Tuple2 (m I64)), p : m (I64 -> I64) };
    type [t : (* -> *) -> *, m : * -> *] Held t m = box struct { o : t (Tuple2 (m (I64 -> I64))) };

    use_fn : I64 -> Outer NW Fn -> I64;
    use_fn = |d, w| if d > 0 { use_fn(d - 1, w) } else { (w.@g)((w.@o.@x.@0.@f)(w.@o.@x.@1)) };

    use_nt : I64 -> Outer NW Nt -> I64;
    use_nt = |d, w| if d > 0 { use_nt(d - 1, w) } else { (w.@p.@v)(w.@o.@x.@0.@v + w.@o.@x.@1) };

    use_held : I64 -> Held NW Nt -> I64;
    use_held = |d, w| if d > 0 { use_held(d - 1, w) } else { (w.@o.@x.@0.@v)(w.@o.@x.@1) };

    main : IO ();
    main = (
        let n = *get_args.map(|args| args.@size);
        let g = if n > 5 { |x| x + n } else { |x| x * 2 + n };
        let with_fn = use_fn(n, Outer { g : g, o : NW { x : (Fn { f : |x| x * 3 + n }, 5) }, p : Fn { f : |h| h } });
        let with_nt = use_nt(n, Outer { g : |x| x, o : NW { x : (Nt { v : 10 + n }, 5) }, p : Nt { v : g } });
        let held = use_held(n, Held { o : NW { x : (Nt { v : |x| x * 7 + n }, 5) } });
        assert_eq(|_| "", [with_fn, with_nt, held], [30 + 3 * n, 30 + 3 * n, 35 + n]);;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_mode());
}

/// Two types referring to each other, each declaring a function field, walked by a recursive
/// function: the copies of the two types share the sets of their fields, and the walk calls the
/// function of each field it reaches.
#[test]
fn test_functions_in_two_types_referring_to_each_other_compute_them() {
    let source = r#"
    module Main;

    type Chain = box union { end : I64 -> I64, link : Link };
    type Link = box struct { step : I64 -> I64, rest : Chain };

    eval_chain : I64 -> Chain -> I64 -> I64;
    eval_chain = |d, c, x| if d > 100 { eval_chain(d - 1, c, x) } else {
        match c {
            end(f) => f(x),
            link(l) => eval_chain(d, l.@rest, (l.@step)(x))
        }
    };

    main : IO ();
    main = (
        let n = *get_args.map(|args| args.@size);
        let c = Chain::link(Link { step : |x| x * 2 + n, rest : Chain::link(Link { step : |x| x + 3, rest : Chain::end(|x| x - n) }) });
        let c2 = if n > 5 { c } else { Chain::end(|x| x * 100) };
        assert_eq(|_| "", [eval_chain(n, c, 5), eval_chain(n, c2, 5)], [13, 500]);;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_mode());
}

/// A type that refers to itself at other type arguments than its own, with a function field
/// typed by the argument: the function of the outer value and that of the inner one have
/// different types, and each call reaches the one stored in the value it reads.
#[test]
fn test_functions_in_a_type_referring_to_itself_at_other_arguments_compute_them() {
    let source = r#"
    module Main;

    type L a = box struct { f : a -> I64, next : Option (L I64) };
    type Alt a b = box struct { here : a -> b, there : Option (Alt b a) };

    main : IO ();
    main = (
        let n = *get_args.map(|args| args.@size);
        let inner : L I64 = L { f : |x| x + n, next : none() };
        let outer : L Bool = L { f : |b| if b { n } else { 3 }, next : some(inner) };
        let r = (outer.@f)(n > 5);
        let s = (outer.@next.as_some.@f)(r);
        let back : Alt Bool I64 = Alt { here : |b| if b { n } else { 7 }, there : none() };
        let forth : Alt I64 Bool = Alt { here : |x| x > n, there : some(back) };
        let t = (forth.@there.as_some.@here)((forth.@here)(n + 1));
        assert_eq(|_| "", [s, t], [3 + n, n]);;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_mode());
}

/// A type that refers to itself at `F64`, with a function field typed by the argument: the
/// function of the outer value takes an integer and that of the inner one a floating-point number,
/// and each is called with its own.
#[test]
fn test_functions_in_a_type_referring_to_itself_at_a_float_argument_compute_them() {
    let source = r#"
    module Main;

    type L a = box struct { f : a -> I64, next : Option (L F64) };

    main : IO ();
    main = (
        let n = *get_args.map(|args| args.@size);
        let inner : L F64 = L { f : |x| (x * 2.5).to_I64 + n, next : none() };
        let outer : L I64 = L { f : |i| i * 3 + n, next : some(inner) };
        let r = (outer.@f)(n + 4);
        let s = (outer.@next.as_some.@f)(r.to_F64 + 0.5);
        assert_eq(|_| "", [r, s], [4 * n + 12, 11 * n + 31]);;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_mode());
}

/// Two types referring to each other, one of them to the other at `String` too, both holding a
/// third type with a function field typed by the argument: the function reached at the argument
/// and the one reached at `String` have different types, and each is called with its own.
#[test]
fn test_functions_in_two_types_referring_to_each_other_at_other_arguments_compute_them() {
    let source = r#"
    module Main;

    type W a = box struct { g : a -> I64 };
    type A x = box struct { w : W x, b : Option (B x) };
    type B y = box struct { a1 : Option (A y), a2 : A String };

    main : IO ();
    main = (
        let n = *get_args.map(|args| args.@size);
        let sa : A String = A { w : W { g : |s| s.get_size + n }, b : none() };
        let ia : A I64 = A { w : W { g : |i| i * 3 + n }, b : some(B { a1 : none(), a2 : sa }) };
        let r = (ia.@w.@g)(n + 4);
        let s = (ia.@b.as_some.@a2.@w.@g)("abc");
        assert_eq(|_| "", [r, s], [4 * n + 12, 3 + n]);;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_mode());
}

/// A type that refers to itself at other type arguments than its own, where the program stores a
/// function in the outer value alone and also calls the field of an inner value, which no value
/// reaches: the one function stored is called with an argument of its own type.
#[test]
fn test_a_function_in_a_type_referring_to_itself_at_other_arguments_beside_a_call_of_another_type_computes_it(
) {
    let source = r#"
    module Main;

    type L a = box struct { f : a -> I64, next : Option (L I64) };

    main : IO ();
    main = (
        let n = *get_args.map(|args| args.@size);
        let outer : L Bool = L { f : |b| if b { n } else { 3 }, next : none() };
        let r = (outer.@f)(n == n);
        let s = match outer.@next { none(_) => 100, some(i) => (i.@f)(r) };
        assert_eq(|_| "", [r, s], [n, 100]);;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_mode());
}

/// A type that refers to itself at other type arguments than its own, whose values two global
/// functions build, where the program calls the function of the inner value alone.
#[test]
fn test_functions_of_two_types_built_into_one_type_by_global_functions_compute_the_one_called() {
    let source = r#"
    module Main;

    type L a = box struct { f : a -> I64, next : Option (L I64) };

    mk_inner : I64 -> L I64;
    mk_inner = |n| L { f : |x| x + n, next : none() };

    mk_outer : I64 -> L Bool;
    mk_outer = |n| L { f : |b| if b { n } else { 3 }, next : some(mk_inner(n)) };

    main : IO ();
    main = (
        let n = *get_args.map(|args| args.@size);
        let outer = mk_outer(n);
        let s = (outer.@next.as_some.@f)(n + 5);
        assert_eq(|_| "", s, 2 * n + 5);;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_mode());
}

/// An unboxed type that refers to itself at other type arguments than its own, whose functions of
/// two types are captured by lambdas a global function calls: `h` captures the function of the
/// inner value, and `k` captures `h` and the outer value, which holds its function in place.
#[test]
fn test_lambdas_capturing_functions_of_a_type_referring_to_itself_at_other_arguments_compute_them()
{
    let source = r#"
    module Main;

    type L a = unbox struct { f : a -> I64, next : Option (Box (L I64)) };

    apply_twice : (I64 -> I64) -> I64 -> I64;
    apply_twice = |f, x| f(f(x));

    main : IO ();
    main = (
        let n = *get_args.map(|args| args.@size);
        let inner : L I64 = L { f : |x| x + n, next : none() };
        let outer : L Bool = L { f : |b| if b { n } else { 3 }, next : some(Box::make(inner)) };
        let g = inner.@f;
        let h = |x| g(x) * 2 + n;
        let k = |x| h((outer.@f)(x > 2)) + (outer.@next.as_some.@value.@f)(1);
        assert_eq(|_| "", [apply_twice(h, 1), apply_twice(k, n)], [9 * n + 4, 6 * n + 1]);;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_mode());
}

/// How many functions of `global_struct_chain_source` take a struct holding a function and hand
/// one to the next.
const GLOBAL_STRUCT_CHAIN_DEPTH: usize = 11;

/// A chain of functions `f{GLOBAL_STRUCT_CHAIN_DEPTH}` to `f0`, each taking its function in a
/// struct field and calling the next from three places: two with a struct holding a lambda of its
/// own wrapping the function it was given, and one with the global struct `gw`. `use_w` is called
/// both with `gw` and with a struct whose function nothing passes down the chain.
fn global_struct_chain_source() -> String {
    let links: Vec<String> = (1..=GLOBAL_STRUCT_CHAIN_DEPTH)
        .map(|i| {
            format!(
                "f{i} : W -> I64 -> I64;\n\
                 f{i} = |w, x| if x % 3 == 0 {{ f{p}(W {{ fn : |y| (w.@fn)(y) + {i}, k : w.@k + 1 }}, x / 3) }} \
                 else if x % 11 == 0 {{ f{p}(gw, x - 1) }} \
                 else {{ f{p}(W {{ fn : |y| (w.@fn)(y) * {i}, k : w.@k }}, x - 1) }};\n",
                i = i,
                p = i - 1
            )
        })
        .collect();
    format!(
        "module Main;\n\
         \n\
         type W = struct {{ fn : I64 -> I64, k : I64 }};\n\
         \n\
         gw : W;\n\
         gw = W {{ fn : |x| x * 3 + 1, k : 5 }};\n\
         \n\
         use_w : W -> I64 -> I64;\n\
         use_w = |w, x| if x < -100000 {{ use_w(w, x + 1) }} else {{ (w.@fn)(x) + w.@k }};\n\
         \n\
         f0 : W -> I64 -> I64;\n\
         f0 = |w, x| (w.@fn)(x) + w.@k;\n\
         {links}\
         \n\
         main : IO ();\n\
         main = (\n\
         \x20   let n = *(pure(100) : IO I64);\n\
         \x20   let v = f{depth}(gw, n) + use_w(gw, n) + use_w(W {{ fn : |x| x - n, k : 1 }}, n)\n\
         \x20       + f{depth}(W {{ fn : |x| x + n, k : 0 }}, n + 2);\n\
         \x20   println(v.to_string)\n\
         );\n",
        links = links.join(""),
        depth = GLOBAL_STRUCT_CHAIN_DEPTH
    )
}

/// The copies of the functions far down `global_struct_chain_source` go past the copy budget, and
/// the sets given to those functions, the set of the function of `gw` among them, are represented
/// by closures. The two calls of `use_w` get copies representing the field differently, and the
/// program computes its value under the development configuration.
#[test]
fn test_a_global_struct_of_a_function_passed_down_a_chain_past_the_copy_budget_computes_its_value()
{
    assert_source_prints(
        &global_struct_chain_source(),
        Configuration::develop_mode(),
        "483",
    );
}

/// The chain of `branching_chain_source`, where each function takes the function and the number in
/// a tuple.
fn tuple_branching_chain_source(depth: usize) -> String {
    chain_of_two_lambda_calls_source(
        depth,
        ChainArgs::Tupled,
        "I64",
        "",
        "g(x)",
        "println(CHAIN_CALL.to_string)",
    )
}

/// The chains of `branching_chain_source`, `returning_branching_chain_source` and
/// `tuple_branching_chain_source` at `BRANCHING_CHAIN_DEPTH` go past the copy budget, and each
/// computes its value under the development configuration.
#[test]
fn test_chains_of_functions_each_calling_the_next_with_two_lambdas_compute_their_values_past_the_copy_budget(
) {
    for (source, expected) in [
        (
            branching_chain_source(BRANCHING_CHAIN_DEPTH),
            BRANCHING_CHAIN_OUTPUT,
        ),
        (
            returning_branching_chain_source(BRANCHING_CHAIN_DEPTH),
            RETURNING_BRANCHING_CHAIN_OUTPUT,
        ),
        (
            tuple_branching_chain_source(BRANCHING_CHAIN_DEPTH),
            BRANCHING_CHAIN_OUTPUT,
        ),
    ] {
        assert_source_prints(&source, Configuration::develop_mode(), expected);
    }
}
