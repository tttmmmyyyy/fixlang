// What a closure stores: the captured values whose type occupies storage. A value of a type that
// occupies none carries no information, so the function a lambda becomes makes one of its own where
// it would have read it out of the capture object, and a closure capturing only such values stores
// nothing and allocates no capture object.

#[cfg(test)]
mod tests {
    use crate::tests::test_util::{
        build_run_and_read_rc_ir, build_within_and_run, rc_ir_function_bodies,
    };
    use std::time::Duration;

    /// Asserts that every closure `dump` builds at the type `closure_ty` stores nothing.
    ///
    /// The type is what names the closure under test: a lambda is lifted into whichever function
    /// ends up using it, so the name it is lifted under says nothing about where it was written.
    /// That a closure of the type is built at all is asserted alongside, so that a build where the
    /// lambda no longer stands as a closure fails here rather than passing on an empty search.
    fn assert_closures_at_type_store_nothing(dump: &str, closure_ty: &str) {
        let held_at = format!(" : {} [", closure_ty);
        let built = dump
            .lines()
            .filter(|line| line.contains("= closure ") && line.contains(&held_at))
            .collect::<Vec<_>>();
        assert!(
            !built.is_empty(),
            "no closure of `{}` is built:\n{}",
            closure_ty,
            dump
        );
        for line in built {
            assert!(
                line.trim_end().ends_with("[]"),
                "a closure of `{}` should store nothing, and this one stores:\n{}",
                closure_ty,
                line
            );
        }
    }

    /// A lambda with no free variable, handed to a built-in that takes a closure. The built-in is
    /// what leaves a closure to look at: a Fix function handed the same lambda is specialized on it,
    /// and then none is built.
    const NO_FREE_VARIABLE: &str = r#"
        module Main;

        _length : Array U8 -> I64;
        _length = |bytes| bytes.borrow_elements(|ptr| FFI_CALL[I64 strlen(Ptr), ptr]);

        main : IO ();
        main = println $ _length("0123456789".get_bytes).to_string;
    "#;

    /// What `NO_FREE_VARIABLE` prints: the length of the string it measures.
    const NO_FREE_VARIABLE_OUTPUT: &str = "10";

    /// A lambda with no free variable stores nothing.
    ///
    /// The lifting that precedes the closure hands such a lambda a capture list of no field, and
    /// that list is a value occupying no storage, so the closure built from it stores nothing. Left
    /// stored, it is one heap allocation per closure built.
    ///
    /// The dump is what this asserts against because the program cannot observe it: the answer is
    /// the same either way.
    #[test]
    fn test_a_lambda_with_no_free_variable_stores_nothing() {
        let dump = build_run_and_read_rc_ir(
            NO_FREE_VARIABLE,
            "max",
            NO_FREE_VARIABLE_OUTPUT,
            "a lambda with no free variable handed to a built-in taking a closure",
        );
        assert_closures_at_type_store_nothing(&dump, "Std::Ptr -> Std::I64");
    }

    /// A lambda that captures a unit and reads it: the pair it answers with carries the unit it
    /// captured, so the body reaches the captured name.
    const CAPTURES_A_UNIT: &str = r#"
        module Main;

        _length_with : () -> Array U8 -> ((), I64);
        _length_with = |unit, bytes| bytes.borrow_elements(
            |ptr| (unit, FFI_CALL[I64 strlen(Ptr), ptr])
        );

        main : IO ();
        main = (
            let (_, length) = _length_with((), "0123456789".get_bytes);
            println $ length.to_string
        );
    "#;

    /// What `CAPTURES_A_UNIT` prints: the length of the string it measures.
    const CAPTURES_A_UNIT_OUTPUT: &str = "10";

    /// A captured value that occupies no storage is read by the body that captured it, and the
    /// closure still stores nothing: the value the body reads is made inside the function the lambda
    /// became, which the dump names `no_storage_value`.
    #[test]
    fn test_a_captured_value_occupying_no_storage_is_made_where_it_is_read() {
        let dump = build_run_and_read_rc_ir(
            CAPTURES_A_UNIT,
            "max",
            CAPTURES_A_UNIT_OUTPUT,
            "a lambda capturing a unit and reading it",
        );
        assert_closures_at_type_store_nothing(&dump, "Std::Ptr -> ((), Std::I64)");

        // The capture list `Main` declares for the lambda is made where it is read, rather than
        // projected out of a capture object the closure would have had to carry it in.
        assert!(
            dump.lines().any(|line| line.contains("Main::#CapList@")
                && line.trim_end().ends_with("= no_storage_value")),
            "the capture list should be made where it is read:\n{}",
            dump
        );
    }

    /// Three lines of output around a call that takes a closure. The `IOState` an `IO` action
    /// threads occupies no storage, so it is among the captures left out of a closure, and what says
    /// it was left out without moving the actions is the order the lines come in.
    const IO_AROUND_A_CLOSURE: &str = r#"
        module Main;

        _length : Array U8 -> I64;
        _length = |bytes| bytes.borrow_elements(|ptr| FFI_CALL[I64 strlen(Ptr), ptr]);

        main : IO ();
        main = (
            println("before");;
            println(_length("0123456789".get_bytes).to_string);;
            println("after")
        );
    "#;

    /// What `IO_AROUND_A_CLOSURE` prints: the three lines in the order they are written.
    const IO_AROUND_A_CLOSURE_OUTPUT: &str = "before\n10\nafter";

    /// The actions of an `IO` keep their order at every optimization level. The `IOState` an `IO`
    /// action threads occupies no storage, so it is among the captures left out of the closures that
    /// captured it, and it is made where it is read instead. What orders the actions is the sequence
    /// of bindings they were lowered into.
    ///
    /// Every level is covered because which captures reach a closure differs by level: below
    /// `-O max` a captured `IOState` is left out as itself, and at `-O max` it has been gathered
    /// into a capture list first.
    #[test]
    fn test_io_keeps_its_order_at_every_optimization_level() {
        for opt_level in ["none", "basic", "max"] {
            assert_eq!(
                build_within_and_run(
                    IO_AROUND_A_CLOSURE,
                    opt_level,
                    Duration::from_secs(600),
                    "three `IO` actions around a call taking a closure",
                ),
                IO_AROUND_A_CLOSURE_OUTPUT,
                "the actions should run in order at -O {}",
                opt_level
            );
        }
    }

    /// A captured `IOState` is left out of the closure and made where it is read.
    ///
    /// The level is `none` because that is where a closure meets an `IOState` as a captured value of
    /// its own: closure specialization, which runs from `-O max` up, gathers a lambda's captures into
    /// one capture list, and that list occupies storage as soon as one of the values in it does.
    #[test]
    fn test_an_iostate_capture_is_left_out_of_the_closure() {
        let dump = build_run_and_read_rc_ir(
            IO_AROUND_A_CLOSURE,
            "none",
            IO_AROUND_A_CLOSURE_OUTPUT,
            "three `IO` actions around a call taking a closure",
        );
        assert!(
            dump.lines()
                .any(|line| line.contains(" : Std::IO::IOState ")
                    && line.trim_end().ends_with("= no_storage_value")),
            "an `IOState` a closure captured should be made where it is read:\n{}",
            dump
        );
    }

    /// A lambda capturing values of both kinds: `u` and `v` occupy no storage, `tag` and `n` occupy
    /// storage, and the body reads all four.
    const MIXED_CAPTURES: &str = r#"
        module Main;

        _describe : () -> String -> () -> I64 -> Array U8 -> ((), String, (), I64, I64);
        _describe = |u, tag, v, n, bytes| bytes.borrow_elements(
            |ptr| (u, tag, v, n, FFI_CALL[I64 strlen(Ptr), ptr])
        );

        main : IO ();
        main = (
            let (_, tag, _, n, length) = _describe((), "tag", (), 7, "0123456789".get_bytes);
            println $ tag + "," + n.to_string + "," + length.to_string
        );
    "#;

    /// What `MIXED_CAPTURES` prints: the two captured values the closure stores, then the length of
    /// the string it measures.
    const MIXED_CAPTURES_OUTPUT: &str = "tag,7,10";

    /// A closure storing some of its captures and leaving the rest out reads every one of them as
    /// the value it was handed: the stored ones are projected at the positions they hold in what the
    /// closure stores, and the rest are made in the function the lambda became.
    ///
    /// The level is `none` for the reason the test above gives: at `-O max` a lambda's captures
    /// arrive as one capture list, so no closure is handed a mixture.
    #[test]
    fn test_a_closure_storing_some_of_its_captures_reads_them_all() {
        let dump = build_run_and_read_rc_ir(
            MIXED_CAPTURES,
            "none",
            MIXED_CAPTURES_OUTPUT,
            "a lambda capturing two values that occupy storage and two that occupy none",
        );
        // A build that stopped handing the closure a mixture fails here, rather than passing on a
        // closure whose captures are all of one kind.
        assert!(
            rc_ir_function_bodies(&dump, "Main::_describe")
                .iter()
                .any(|body| body.contains("= capture_project_")
                    && body.contains("= no_storage_value")),
            "a closure of `Main::_describe` should both project a capture and make one:\n{}",
            dump
        );
    }
}
