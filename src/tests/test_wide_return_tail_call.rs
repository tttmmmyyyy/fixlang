use crate::{configuration::Configuration, tests::test_util::test_source};

// `Std::IO`'s `bind` puts `(f(a).@runner)(iostate)` in tail position, so a monadic loop is a chain
// of indirect tail calls and runs in constant stack only while the backend compiles them as jumps.
// Two properties of the signature decide whether it does, and `return_abi` handles both: the result
// must fit in the return registers, and the arguments whose values change must fit in the argument
// registers. An `Array` occupies three scalar leaves (storage, size, capacity), so an ordinary
// monadic result crosses the x86-64 budget of three integer registers.
//
// Each test below drives one such loop a million iterations deep and checks only that it finishes.
// The outcome is binary — completes or overflows the stack — so machine load does not affect it.
// They run at every optimization level: whether a tail call becomes a jump is decided by the
// backend, which does it at `-O0` too, and by the `tail` marker, which code generation attaches to
// every call in tail position.
//
// AArch64 returns up to eight leaves in registers, which covers the four-leaf shapes; the shape that
// exercises the return rule on every target is
// `test_return_wider_than_any_target_runs_in_constant_stack`.

// The recursive call sits in tail position of a bind's continuation, and the result is an `Array`
// plus a scalar. This is the shape a monadic loop over a growing or threaded array takes.
#[test]
fn test_monadic_loop_with_array_result_runs_in_constant_stack() {
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
// optimization level. Inlining and closure specialization cannot fold this chain into a
// self-recursive loop;
// only turning the tail call into a jump keeps the stack flat.
#[test]
fn test_dispatch_through_array_runs_in_constant_stack() {
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

// The two functions carry different numbers of arguments, so the call from the narrow one to the wide
// one has to grow the outgoing argument area. A sibcall may only reuse the caller's own argument
// area, while a tail call under `tailcc` may grow it, which is what keeps this loop flat. Ten
// arguments overflow the argument registers of both supported targets, and stay under the arity where
// the compiler's own eta expansion blows up (fixlang issue #76).
//
// x86-64 alone, since that is where `lambda_calling_convention_of_target` gives Fix lambdas `tailcc`.
// AArch64 keeps the C convention, where this call stays an ordinary one and the stack grows with the
// recursion (fixlang issue #111).
#[cfg(target_arch = "x86_64")]
#[test]
fn test_growing_argument_area_mutual_recursion_runs_in_constant_stack() {
    let source = r#"
    module Main;

    narrow : I64 -> I64 -> I64 -> I64 -> I64 -> I64 -> I64;
    narrow = |n, a, b, c, d, e| (
        if n == 0 { a + b + c + d + e };
        wide(n - 1, a, b, c, d, e, a + 1, b + 1, c + 1, d + 1)
    );

    wide : I64 -> I64 -> I64 -> I64 -> I64 -> I64 -> I64 -> I64 -> I64 -> I64 -> I64;
    wide = |n, a, b, c, d, e, f, g, h, i| (
        if n == 0 { a + b + c + d + e + f + g + h + i };
        narrow(n - 1, a, b, c, d, e)
    );

    main : IO ();
    main = (
        let r = narrow(1000000, 1, 2, 3, 4, 5);
        assert_eq(|_|"unexpected result", r, 15);;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_mode());
}

// The result crosses the floating-point half of the register budget rather than the integer half.
// `tailcc`, the convention x86-64 Fix lambdas use, returns five floating-point leaves in registers
// and sends six to memory, so six is where the `float` entry of the budget decides the outcome.
// Every other loop here carries pointers and integers only.
#[test]
fn test_float_wide_return_runs_in_constant_stack() {
    let source = r#"
    module Main;

    walk : I64 -> (F64, F64, F64, F64, F64, F64) -> IO (F64, F64, F64, F64, F64, F64);
    walk = |i, st| (
        if i == 0 { pure(st) };
        let _ = *pure(0);
        walk(i - 1, (st.@0 + 1.0, st.@1, st.@2, st.@3, st.@4, st.@5))
    );

    main : IO ();
    main = (
        let (a, b, c, d, e, f) = *walk(1000000, (0.0, 1.0, 2.0, 3.0, 4.0, 5.0));
        assert_eq(|_|"unexpected result", a + b + c + d + e + f, 1000015.0);;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_mode());
}

// Under separated compilation the unit that defines a function and the unit that calls it are
// generated apart, so both sides derive the out-pointer decision and the calling convention from the
// target alone. One symbol per unit puts a unit boundary on every call. The shapes cover an integer
// result wider than the return registers, a result carrying arrays, a result mixing floating-point
// and integer leaves, a wide global, and mutual recursion between two wide-returning functions.
#[test]
fn test_wide_return_across_compilation_units() {
    let source = r#"
    module Main;

    mk4 : I64 -> (I64, I64, I64, I64);
    mk4 = |n| (n, n + 1, n + 2, n + 3);

    mk_arrays : I64 -> (Array I64, Array I64);
    mk_arrays = |n| (Array::fill(n, 1), Array::fill(n + 1, 2));

    mixed : I64 -> (F64, F64, I64, I64, F64);
    mixed = |n| (n.f64, (n + 1).f64, n + 2, n + 3, (n + 4).f64);

    wide_const : (I64, I64, I64, I64, I64);
    wide_const = (11, 22, 33, 44, 55);

    step_a : I64 -> (I64, I64, I64, I64);
    step_a = |n| if n == 0 { (1, 2, 3, 4) } else { step_b(n - 1) };

    step_b : I64 -> (I64, I64, I64, I64);
    step_b = |n| if n == 0 { (5, 6, 7, 8) } else { step_a(n - 1) };

    main : IO ();
    main = (
        let (a, b, c, d) = mk4(10);
        assert_eq(|_|"mk4", a * 1000 + b * 100 + c * 10 + d, 10 * 1000 + 11 * 100 + 12 * 10 + 13);;
        let (xs, ys) = mk_arrays(3);
        assert_eq(|_|"mk_arrays", xs.@size * 10 + ys.@size, 34);;
        let (u, v, w, x, y) = mixed(2);
        assert_eq(|_|"mixed", (u + v + y).i64 + w + x, 2 + 3 + 6 + 4 + 5);;
        let (p, q, r, s, t) = wide_const;
        assert_eq(|_|"wide_const", p + q + r + s + t, 165);;
        let (e, f, g, h) = step_a(7);
        assert_eq(|_|"step", e + f + g + h, 5 + 6 + 7 + 8);;
        pure()
    );
    "#;
    let mut config = Configuration::develop_mode();
    config.max_cu_size = 1;
    test_source(source, config);
}

// The out-pointer path must move the same bytes the register path did, for every leaf shape that
// straddles the budget: three integer leaves against four, a union payload plus its tag, floating-
// point leaves mixed with integer ones, a nested tuple with zero-sized members, and a boxed value.
// Each is reached directly, in tail position, through a closure, and out of an array, so a value
// crosses the out-pointer both as a forwarded parameter and as a caller-side buffer.
#[test]
fn test_wide_return_shapes_compute_correctly() {
    let source = r#"
    module Main;

    type Rec = unbox struct { a : I64, b : F64, c : F32, d : U8 };
    type Boxy = box struct { p : I64, q : I64, r : I64, s : I64 };

    // Three integer leaves: within the x86-64 return registers.
    narrow : I64 -> (I64, I64, I64);
    narrow = |n| (n, n + 1, n + 2);

    // Four integer leaves: past them.
    wide : I64 -> (I64, I64, I64, I64);
    wide = |n| (n, n + 1, n + 2, n + 3);

    // Integer and floating-point leaves mixed, and a nested tuple with zero-sized members.
    mixed : I64 -> (I64, (), F64, ((), F32), U8, ());
    mixed = |n| (n, (), n.f64 + 0.5, ((), n.f32 + 0.25_F32), (n + 7).u8, ());

    // A union payload plus its tag.
    unioned : I64 -> Result String I64;
    unioned = |n| if n % 2 == 0 { err("e" + n.to_string) } else { ok(n * 3) };

    // A struct of several scalar kinds, and a boxed one (a single pointer leaf).
    record : I64 -> Rec;
    record = |n| Rec { a : n, b : n.f64 + 0.5, c : n.f32 + 0.25_F32, d : (n + 7).u8 };

    boxed : I64 -> Boxy;
    boxed = |n| Boxy { p : n, q : n + 1, r : n + 2, s : n + 3 };

    chk : I64 -> I64;
    chk = |n| (
        let t3 = narrow(n);
        let t4 = wide(n);
        let m = mixed(n);
        let u = unioned(n);
        let r = record(n);
        let b = boxed(n);
        t3.@0 + t3.@1 * 3 + t3.@2 * 5
            + t4.@0 * 7 + t4.@1 * 11 + t4.@2 * 13 + t4.@3 * 17
            + m.@0 * 19 + (m.@2 * 2.0).i64 * 23 + (m.@3.@1 * 4.0_F32).i64 * 29
            + m.@4.i64 * 31
            + (if u.is_ok { u.as_ok } else { -u.as_err.@size }) * 37
            + r.@a * 41 + (r.@b * 2.0).i64 * 43 + (r.@c * 4.0_F32).i64 * 47
            + r.@d.i64 * 53
            + b.@p * 59 + b.@q * 61 + b.@r * 67 + b.@s * 71
    );

    // The same functions reached in tail position, through a closure, and out of an array, so the
    // value crosses the out-pointer both as a forwarded parameter and as a caller-side buffer.
    tail_wide : I64 -> (I64, I64, I64, I64);
    tail_wide = |n| wide(n);

    table : Array (I64 -> (I64, I64, I64, I64));
    table = [wide, tail_wide, |n| wide(n + 100)];

    indirect : I64 -> I64;
    indirect = |n| (
        let k = |m| wide(m + 1000);
        let a = k(n);
        let b = (table.@(n % 3))(n);
        let c = tail_wide(n);
        a.@0 + a.@3 * 3 + b.@0 * 5 + b.@3 * 7 + c.@0 * 11 + c.@3 * 13
    );

    main : IO ();
    main = (
        let acc = Iterator::range(0, 20).fold(0, |n, acc| acc * 3 + chk(n) + indirect(n) * 97);
        assert_eq(|_|"unexpected result", acc, 710000889699219);;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_mode());
}

// Three integer and four floating-point leaves fill both halves of the budget at once, the widest
// result the table still returns in registers on x86-64.
#[test]
fn test_mixed_result_filling_both_register_classes_runs_in_constant_stack() {
    let source = r#"
    module Main;

    type S = (I64, I64, I64, F64, F64, F64, F64);

    walk : I64 -> S -> IO S;
    walk = |i, st| (
        if i == 0 { pure(st) };
        let _ = *pure(0);
        walk(i - 1, (st.@1, st.@2, st.@0 + 1, st.@4, st.@5, st.@6, st.@3 + 1.0))
    );

    main : IO ();
    main = (
        let (a, b, c, d, e, f, g) = *walk(1000000, (0, 1, 2, 0.0, 1.0, 2.0, 3.0));
        assert_eq(|_|"unexpected integer result", a + b + c, 1000003);;
        assert_eq(|_|"unexpected float result", d + e + f + g, 1000006.0);;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_mode());
}

// A `Destructor` holding a wide resource. Its destructor is applied from inside the reference-
// counting helper that releases the value -- a function with the C convention, not a Fix lambda -- so
// that helper's own frame holds the out-pointer buffer of the call. It is the one place outside a
// Fix lambda where a buffer is allocated and handed to a call, which is what makes the call's
// `tail` marker load-bearing there.
#[test]
fn test_destructor_with_wide_resource_is_memory_safe() {
    let source = r#"
    module Main;

    main : IO ();
    main = (
        let d = *Destructor::make(
            (Array::fill(3, "p"), Array::fill(3, "q"), Array::fill(3, "r")),
            |v| pure(v)
        );
        let n = d.borrow(|v| v.@0.@size + v.@1.@size + v.@2.@size);
        assert_eq(|_|"unexpected result", n, 9);;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_mode());
}
