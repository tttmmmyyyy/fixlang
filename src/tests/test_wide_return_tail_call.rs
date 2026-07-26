use crate::{
    configuration::Configuration,
    tests::test_util::{tail_call_optimization_enabled, test_source},
};

// `Std::IO`'s `bind` puts `(f(a).@runner)(iostate)` in tail position, so a monadic loop is a chain
// of indirect tail calls and runs in constant stack only while the backend compiles them as jumps.
// Two properties of the signature decide whether it does, and `return_abi` handles both: the result
// must fit in the return registers, and the arguments whose values change must fit in the argument
// registers. An `Array` occupies three scalar leaves (storage, size, capacity), so an ordinary
// monadic result crosses the x86-64 budget of three integer registers.
//
// Each test below drives one such loop a million iterations deep and checks only that it finishes.
// The outcome is binary — completes or overflows the stack — so machine load does not affect it.
//
// AArch64 returns up to eight leaves in registers, which covers the four-leaf shapes; the shape that
// exercises the return rule on every target is
// `test_return_wider_than_any_target_runs_in_constant_stack`.

// The recursive call sits in tail position of a bind's continuation, and the result is an `Array`
// plus a scalar. This is the shape a monadic loop over a growing or threaded array takes.
#[test]
fn test_monadic_loop_with_array_result_runs_in_constant_stack() {
    if !tail_call_optimization_enabled() {
        return;
    }
    let source = r#"
    module Main;

    walk : I64 -> (Array String, I64) -> IO (Array String, I64);
    walk = |i, st| (
        if i == 0 { pure(st) };
        let _ = *pure(0);
        walk(i - 1, (st.@0, st.@1 + 1))
    );

    main : IO ();
    main = (
        let (xs, acc) = *walk(1000000, (Array::fill(4, "x"), 0));
        assert_eq(|_|"unexpected result", acc + xs.@size, 1000004);;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_mode());
}

// `IOFail a` wraps `IO (Result ErrMsg a)`, and `ErrMsg` is a `String`, so the result is wide for
// every element type. A loop in `IOFail` therefore needs the rule even when it threads no array.
#[test]
fn test_iofail_loop_runs_in_constant_stack() {
    if !tail_call_optimization_enabled() {
        return;
    }
    let source = r#"
    module Main;

    walk : I64 -> I64 -> IOFail I64;
    walk = |i, acc| (
        if i == 0 { pure(acc) };
        let _ = *pure(0);
        walk(i - 1, acc + 1)
    );

    main : IO ();
    main = (
        let res = *walk(1000000, 0).to_result;
        assert_eq(|_|"unexpected result", res.as_ok, 1000000);;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_mode());
}

// `Std::loop_m` recurses on itself in tail position of a bind, so its own return width is what
// decides whether the loop is constant-stack. Here `break_m` carries an array and a scalar.
#[test]
fn test_loop_m_with_wide_break_runs_in_constant_stack() {
    if !tail_call_optimization_enabled() {
        return;
    }
    let source = r#"
    module Main;

    main : IO ();
    main = (
        let (xs, acc) = *loop_m((Array::fill(4, "x"), 0, 0), |(xs, acc, i)|
            if i == 1000000 { break_m $ (xs, acc) };
            continue_m $ (xs, acc + 1, i + 1)
        );
        assert_eq(|_|"unexpected result", acc + xs.@size, 1000004);;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_mode());
}

// A user-defined monad transformer stacked on `IO`, the form a state-carrying library monad takes.
// Its `bind` ends with `((f(a)).@run)(s)`, so the transformer's own return width — the state plus
// the value — decides the stack behavior of a loop written in it.
#[test]
fn test_state_transformer_over_io_runs_in_constant_stack() {
    if !tail_call_optimization_enabled() {
        return;
    }
    let source = r#"
    module Main;

    type [m : *->*] StateT s m a = unbox struct { run : s -> m (s, a) };

    impl [m : Monad] StateT s m : Monad {
        pure = |v| StateT { run : |s| Monad::pure $ (s, v) };
        bind = |f, x| StateT { run : |s|
            let (s, a) = *(x.@run)(s);
            ((f(a)).@run)(s)
        };
    }

    get_state : [m : Monad] StateT s m s;
    get_state = StateT { run : |s| Monad::pure $ (s, s) };

    put_state : [m : Monad] s -> StateT s m ();
    put_state = |s| StateT { run : |_| Monad::pure $ (s, ()) };

    walk : I64 -> StateT (Array String, I64) IO ();
    walk = |i| (
        if i == 0 { pure() };
        let st = *get_state;
        put_state((st.@0, st.@1 + 1));;
        walk(i - 1)
    );

    main : IO ();
    main = (
        let (st, _) = *(walk(1000000).@run)((Array::fill(4, "x"), 0));
        assert_eq(|_|"unexpected result", st.@1 + st.@0.@size, 1000004);;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_mode());
}

// A state monad with no inner monad: its `run` carries the state in the tail call's arguments rather
// than in a capture, so seven state leaves put the call at nine arguments — past both the return
// registers and the six changing arguments an x86-64 sibcall can rewrite under the C convention.
#[test]
fn test_state_monad_carrying_state_in_arguments_runs_in_constant_stack() {
    if !tail_call_optimization_enabled() {
        return;
    }
    let source = r#"
    module Main;

    type State s a = unbox struct { run : s -> (s, a) };

    impl State s : Monad {
        pure = |v| State { run : |s| (s, v) };
        bind = |f, x| State { run : |s|
            let (s, a) = (x.@run)(s);
            ((f(a)).@run)(s)
        };
    }

    get_state : State s s;
    get_state = State { run : |s| (s, s) };

    put_state : s -> State s ();
    put_state = |s| State { run : |_| (s, ()) };

    walk : I64 -> State (Array String, Array String, I64) ();
    walk = |i| (
        if i == 0 { pure() };
        let (xs, ys, acc) = *get_state;
        put_state((xs, ys, acc + 1));;
        walk(i - 1)
    );

    main : IO ();
    main = (
        let init = (Array::fill(4, "x"), Array::fill(5, "y"), 0);
        let ((xs, ys, acc), _) = (walk(1000000).@run)(init);
        assert_eq(|_|"unexpected result", acc + xs.@size + ys.@size, 1000009);;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_mode());
}

// The function called in tail position comes out of an array, so the call stays indirect at every
// optimization level. Inlining and decapturing cannot fold this chain into a self-recursive loop;
// only turning the tail call into a jump keeps the stack flat.
#[test]
fn test_dispatch_through_array_runs_in_constant_stack() {
    if !tail_call_optimization_enabled() {
        return;
    }
    let source = r#"
    module Main;

    table : Array (I64 -> (Array String, I64) -> IO (Array String, I64));
    table = [step_a, step_b];

    step_a : I64 -> (Array String, I64) -> IO (Array String, I64);
    step_a = |i, st| (
        if i == 0 { pure(st) };
        let _ = *pure(0);
        let next = table.@(i % 2);
        next(i - 1, (st.@0, st.@1 + 1))
    );

    step_b : I64 -> (Array String, I64) -> IO (Array String, I64);
    step_b = |i, st| (
        if i == 0 { pure(st) };
        let _ = *pure(0);
        let next = table.@((i + 1) % 2);
        next(i - 1, (st.@0, st.@1 + 1))
    );

    main : IO ();
    main = (
        let (xs, acc) = *step_a(1000000, (Array::fill(4, "x"), 0));
        assert_eq(|_|"unexpected result", acc + xs.@size, 1000004);;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_mode());
}

// Three arrays make nine leaves, above the largest return-register budget among supported targets,
// so this loop needs the out-pointer on AArch64 as well as on x86-64.
#[test]
fn test_return_wider_than_any_target_runs_in_constant_stack() {
    if !tail_call_optimization_enabled() {
        return;
    }
    let source = r#"
    module Main;

    type Three = (Array String, Array String, Array String);

    walk : I64 -> Three -> IO Three;
    walk = |i, st| (
        if i == 0 { pure(st) };
        let _ = *pure(0);
        walk(i - 1, (st.@1, st.@2, st.@0))
    );

    main : IO ();
    main = (
        let init = (Array::fill(4, "x"), Array::fill(5, "y"), Array::fill(6, "z"));
        let (xs, ys, zs) = *walk(1000000, init);
        assert_eq(|_|"unexpected result", xs.@size + ys.@size + zs.@size, 15);;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_mode());
}
