use crate::{configuration::Configuration, tests::test_util::test_source};

// Programs whose function values take the paths the lambda set inference follows: through a
// recursion, into a value that captures a value of its own set, through a `fix` left as a closure,
// and through the fields and the state of a loop. Each runs under the development configuration,
// so the inference runs on it and the result is checked under valgrind.

// A recursion that passes itself a lambda wrapping the function it was given: the lambda at depth
// `n` captures the one at depth `n - 1`.
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

// Folding an array of functions into their composition: the state captures the state of the step
// before, as long a chain as the array.
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

// A `fix` whose argument reaches it through a parameter, so that `defunctionalize_fix` leaves the
// combinator in place.
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

// A lambda the callee chooses in place of the one it was given, joining it at an `if`.
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

// A struct carrying its operation in a field, rebuilt by a setter, chosen at an `if` and carried as
// the state of a loop, as a segment tree carries its monoid.
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
