use crate::{configuration::Configuration, tests::test_util::test_source};

// A game's main loop, in the shape Fix lets one write it: `main` re-enters `main` until a deadline
// passes, threading no state and using no loop combinator. `Std::IO`'s `bind` leaves
// `(f(a).@runner)(iostate)` in tail position, so the self-entry is a tail call that has to lower to
// a jump; a frame kept per iteration overflows the stack long before the deadline.
//
// The deadline is CPU time consumed, which advances at the same rate however loaded the machine is,
// so the iteration count reached does not depend on the tests running beside this one. Ten seconds
// buys tens of millions of iterations natively and millions under valgrind, both far past the
// hundred thousand or so frames an 8 MiB stack holds — a lost tail call therefore ends the process
// on a signal rather than at the deadline, whatever the machine's speed.

/// Verifies that a `main` whose only loop is a self-entry runs in constant stack and exits normally.
#[test]
fn test_main_self_entry_runs_in_constant_stack() {
    let source = r#"
    module Main;

    // Seconds of CPU time this process has consumed since it started.
    process_time : IO F64;
    process_time = (
        let clocks = *FFI_CALL_IO[I64 fixruntime_clock()];
        pure $ FFI_CALL[F64 fixruntime_clocks_to_sec(I64), clocks]
    );

    main : IO ();
    main = (
        let t = *process_time;
        if t < 10.0 { main } else { pure() }
    );
    "#;
    test_source(source, Configuration::develop_mode());
}
