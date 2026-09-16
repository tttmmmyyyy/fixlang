// What a closure stores: the captured values whose type occupies storage. A value of a type that
// occupies none carries no information, so the function a lambda becomes makes one of its own where
// it would have read it out of the capture object, and a closure capturing only such values stores
// nothing and allocates no capture object.

#[cfg(test)]
mod tests {
    use crate::tests::test_util::build_run_and_read_rc_ir;

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

    /// The actions of an `IO` keep their order when the `IOState` they thread is left out of the
    /// closures that captured it. The token carries no information and is made where it is read, so
    /// what orders the actions is the sequence of bindings they were lowered into.
    #[test]
    fn test_io_keeps_its_order_when_the_iostate_is_left_out_of_a_closure() {
        build_run_and_read_rc_ir(
            IO_AROUND_A_CLOSURE,
            "max",
            IO_AROUND_A_CLOSURE_OUTPUT,
            "three `IO` actions around a call taking a closure",
        );
    }
}
