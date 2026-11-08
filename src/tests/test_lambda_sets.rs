use crate::{configuration::Configuration, tests::test_util::test_source};

// Programs whose function values take the paths the lambda set inference follows: through a
// recursion, into a value that captures a value of its own set, through a `fix` left as a closure,
// and through the fields and the state of a loop. Each runs under the development configuration,
// so the inference runs on it and the result is checked under valgrind.

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
/// recursion's copy and holds one lambda in the direct call's, while both copies give the lambda's
/// parameter a set of closures.
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
/// one set of closures, so that `g`'s set holds one lambda, and the other gives them two, so that
/// it holds two, while both represent the parameters alike.
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
/// lambda in the first copy, and in the second two elements whose captured sets are represented by
/// closures and differ in what they hold. The second set is represented by closures, and its
/// elements share the global function and the capture struct of the first set's class, as a
/// closure has one type whatever its set holds. Reaching the second copy through `w2` puts its sets
/// before the first's, so the lambda's class is first met where `g`'s set holds two elements.
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

/// A recursion carrying in an `Option` a function `mk` builds from one of two functions whose sets
/// hold Dynamic: the set of the carried function holds `mk`'s lambda twice, two elements of one
/// class, and each captures in place, through the `Option`, a function of that set again.
#[test]
fn test_two_elements_of_one_class_capturing_an_optional_function_of_their_own_set_compute_the_chain(
) {
    let source = r#"
    module Main;

    fixed : ((I64 -> I64) -> I64 -> I64) -> I64 -> I64;
    fixed = |step| fix(step);

    mk : I64 -> (I64 -> I64) -> Option (I64 -> I64) -> Option (I64 -> I64);
    mk = |d, g, o| if d > 100 { mk(d - 1, g, o) } else {
        some(|y| g(y) + match o { some(f) => f(y), none(_) => 0 })
    };

    rec : I64 -> (I64 -> I64) -> (I64 -> I64) -> Option (I64 -> I64) -> I64 -> I64;
    rec = |n, g1, g2, o, x| if n == 0 { match o { some(f) => f(x), none(_) => x } } else {
        let o2 = if n % 2 == 0 { mk(n, g1, o) } else { mk(n, g2, o) };
        rec(n - 1, g1, g2, o2, x)
    };

    main : IO ();
    main = (
        let n = *get_args.map(|args| args.@size);
        let g1 = if n > 0 { fixed(|loop, i| if i <= 0 { 0 } else { i + loop(i - 1) }) } else { |x| x + n };
        let g2 = if n > 0 { fixed(|loop, i| if i <= 0 { 1 } else { i * loop(i - 1) }) } else { |x| x * n };
        assert_eq(|_|"unexpected result", rec(4, g1, g2, none(), 3), 24);;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_mode());
}

/// What the rewrite by the lambda sets makes of a program, read off the `--emit-rc-ir` dump: a
/// function given a lambda calls it by name, and the lambda's capture becomes a struct the function
/// takes as an argument. The program cannot observe either one — the answer is the same with
/// function values left as closures — so these assert against the dump.
#[cfg(test)]
mod dump_tests {
    use crate::constants::{LS_CAP_PREFIX, LS_LAM_SUFFIX};
    use crate::tests::test_util::{
        build_run_and_read_rc_ir, rc_ir_callee_of, rc_ir_function_bodies,
    };

    /// The function values that `body`, a function of an RC IR dump, calls: each call whose callee
    /// is a local value. A global is written with its namespace, and a builtin operation's name
    /// carries no `#`, which every local name of the dump does.
    fn closure_calls(body: &str) -> Vec<&str> {
        body.lines()
            .filter_map(|line| line.split_once("] = "))
            .filter_map(|(_, rhs)| rhs.split_once('('))
            .map(|(callee, _)| callee)
            .filter(|callee| callee.contains('#') && !callee.contains("::"))
            .collect()
    }

    /// Asserts that `dump` holds functions named with `name_part`, and that none of them builds a
    /// closure or calls one: every function value they call is called by name.
    fn assert_calls_by_name(dump: &str, name_part: &str) {
        let bodies = rc_ir_function_bodies(dump, name_part);
        assert!(
            !bodies.is_empty(),
            "the dump should hold functions named with `{}`, and it holds none",
            name_part
        );
        for body in bodies {
            assert!(
                !body.contains("= closure ") && closure_calls(&body).is_empty(),
                "every function named with `{}` should call the function values it calls by \
                 name, and this one goes through a closure:\n{}",
                name_part,
                body
            );
        }
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

    /// The capture structs among the parameters of the function header `header`, by the name of
    /// their type.
    fn capture_struct_parameters(header: &str) -> Vec<&str> {
        parameters(header)
            .split(|c: char| c.is_whitespace() || c == ',' || c == '(' || c == ')')
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
    /// same fold calls the lambda through a closure, which is what says the assertion can see one.
    #[test]
    fn test_a_function_given_a_lambda_calls_it_by_name() {
        let dump =
            build_run_and_read_rc_ir(FOLD_SOURCE, "max", FOLD_OUTPUT, "a fold given a lambda");
        assert_calls_by_name(&dump, "Std::Iterator::fold");

        let dump =
            build_run_and_read_rc_ir(FOLD_SOURCE, "basic", FOLD_OUTPUT, "a fold given a lambda");
        assert!(
            rc_ir_function_bodies(&dump, "Std::Iterator::fold")
                .iter()
                .any(|body| !closure_calls(body).is_empty()),
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

    main : IO ();
    main = (
        let values = [0, 1, 4, 9, 3, 12, 10, 10, 12, 3];
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
    /// call goes to the callee's borrow version, so the round counts no reference for it.
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

    /// The functions an iterator carries in its fields reach the fold through the iterator's type,
    /// so the fold takes the capture struct of each, three with its own operation, and calls each
    /// by name.
    #[test]
    fn test_a_fold_takes_the_functions_a_chain_of_iterators_carries_as_capture_structs() {
        let dump = build_run_and_read_rc_ir(
            NESTED_ITERATORS_SOURCE,
            "max",
            NESTED_ITERATORS_OUTPUT,
            "a fold over two maps",
        );
        assert_calls_by_name(&dump, "Std::Iterator::fold");
        let most = rc_ir_function_bodies(&dump, "Std::Iterator::fold")
            .iter()
            .map(|body| capture_struct_parameters(header(body)).len())
            .max()
            .unwrap();
        assert!(
            most >= 3,
            "the fold should take the capture structs of the two functions the chain carries and \
             of its own operation, and the most one of its functions takes is {}",
            most
        );
    }

    /// The functions the chain of iterators carries have their bodies put where the fold calls
    /// them: none stands as a function of its own. Each is lifted out of `main`, takes an `I64` and
    /// answers one, which tells it from the IO plumbing lifted out of `main` as well.
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

    /// `k` captures `g`, which `mk` builds from one of two functions whose sets hold Dynamic, so the
    /// set of `g` holds `mk`'s lambda twice, two elements that differ only in sets of closures.
    const TWO_ALIKE_ELEMENTS_SOURCE: &str = r#"
    module Main;

    fixed : ((I64 -> I64) -> I64 -> I64) -> I64 -> I64;
    fixed = |step| fix(step);

    mk : I64 -> (I64 -> I64) -> I64 -> I64;
    mk = |d, f| if d > 100 { mk(d - 1, f) } else { |x| f(x) + 1 };

    k : I64 -> (I64 -> I64) -> (I64 -> I64) -> I64 -> I64;
    k = |d, f1, f2| if d > 100 { k(d - 1, f1, f2) } else {
        let g = if d == 1 { mk(d, f1) } else { mk(d, f2) };
        |y| g(y) + d
    };

    main : IO () = (
        let n = *get_args.map(|args| args.@size);
        let h1 = if n > 0 { fixed(|loop, i| if i <= 0 { 0 } else { i + loop(i - 1) }) } else { |x| x + n };
        let h2 = if n > 0 { fixed(|loop, i| if i <= 0 { 1 } else { i * loop(i - 1) }) } else { |x| x * n };
        println((k(n, h1, h2)(4) + k(n + 1, h1, h2)(4)).to_string)
    );
    "#;

    /// What `TWO_ALIKE_ELEMENTS_SOURCE` prints: `k(1, ..)` adds 2 to the sum of `1..4`, and
    /// `k(2, ..)` adds 3 to the factorial of 4.
    const TWO_ALIKE_ELEMENTS_OUTPUT: &str = "39";

    /// A set whose elements make one class is represented by that class's capture struct, however
    /// many elements it holds: `mk` builds `g` as a capture struct, and no closure.
    #[test]
    fn test_a_set_of_two_elements_of_one_class_is_a_capture_struct() {
        let dump = build_run_and_read_rc_ir(
            TWO_ALIKE_ELEMENTS_SOURCE,
            "max",
            TWO_ALIKE_ELEMENTS_OUTPUT,
            "a lambda capturing a function built from one of two alike functions",
        );
        let bodies = rc_ir_function_bodies(&dump, "Main::mk");
        assert!(
            !bodies.is_empty(),
            "the dump should hold functions named with `Main::mk`, and it holds none"
        );
        for body in bodies {
            assert!(
                !body.contains("= closure "),
                "`mk` should build `g` as a capture struct, and this function builds a closure:\n{}",
                body
            );
        }
    }
}
