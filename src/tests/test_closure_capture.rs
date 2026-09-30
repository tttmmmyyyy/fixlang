//! What a closure stores: the captured values whose type occupies storage. A value of a type that
//! occupies none carries no information, so the function a lambda becomes makes one of its own where
//! it would have read it out of the capture object, and a closure capturing only such values stores
//! nothing and allocates no capture object.

#[cfg(test)]
mod tests {
    use crate::{
        configuration::{Configuration, FixOptimizationLevel, ValgrindTool},
        tests::test_util::{
            build_run_and_read_rc_ir, build_within_and_run, rc_ir_function_bodies, test_source,
        },
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

    /// Asserts that `dump` binds a value made on the spot -- the right-hand side is
    /// `no_storage_value` -- on a line containing `line_part`, rather than projecting that value
    /// out of a capture object.
    ///
    /// `line_part` is matched against the whole line, so it reaches both the type a binding is held
    /// at and the name it was minted under.
    fn assert_a_value_is_made_where_it_is_read(dump: &str, line_part: &str) {
        assert!(
            dump.lines()
                .any(|line| line.contains(line_part)
                    && line.trim_end().ends_with("= no_storage_value")),
            "a value named by `{}` should be made where it is read:\n{}",
            line_part,
            dump
        );
    }

    /// A lambda with no free variable, handed to a built-in that stores it as a closure. The
    /// built-in is what leaves a closure to look at: a Fix function handed the same lambda is
    /// specialized on it, and a built-in that applies its function in a scope takes the capture list
    /// through its environment, and then none is built.
    const NO_FREE_VARIABLE_SOURCE: &str = r#"
        module Main;

        _make_destructor : I64 -> IO (Destructor I64);
        _make_destructor = |n| Destructor::make(n, |m| pure $ m + 1);

        main : IO ();
        main = (
            let dtor = *_make_destructor(9);
            println $ dtor.borrow(|n| n).to_string
        );
    "#;

    /// What `NO_FREE_VARIABLE_SOURCE` prints: the number the destructor holds.
    const NO_FREE_VARIABLE_OUTPUT: &str = "9";

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
            NO_FREE_VARIABLE_SOURCE,
            "max",
            NO_FREE_VARIABLE_OUTPUT,
            "a lambda with no free variable handed to a built-in taking a closure",
        );
        assert_closures_at_type_store_nothing(
            &dump,
            "Std::I64 -> Std::IO::IOState -> (Std::IO::IOState, Std::I64)",
        );
    }

    /// A lambda that captures a unit and reads it: the pair it answers with carries the unit it
    /// captured, so the body reaches the captured name. `Destructor::make` stores the lambda as a
    /// closure, where a built-in that applies its function takes what the lambda captures through
    /// an environment instead.
    const CAPTURES_A_UNIT_SOURCE: &str = r#"
        module Main;

        _keeping_unit : () -> I64 -> IO (Destructor ((), I64));
        _keeping_unit = |unit, n| Destructor::make(((), n), |(_, m)| pure $ (unit, m + 1));

        main : IO ();
        main = (
            let dtor = *_keeping_unit((), 9);
            println $ dtor.borrow(|(_, n)| n).to_string
        );
    "#;

    /// What `CAPTURES_A_UNIT_SOURCE` prints: the number the destructor holds.
    const CAPTURES_A_UNIT_OUTPUT: &str = "9";

    /// A captured value that occupies no storage is read by the body that captured it, and the
    /// closure still stores nothing: the value the body reads is made inside the function the lambda
    /// became, which the dump names `no_storage_value`.
    #[test]
    fn test_a_captured_value_occupying_no_storage_is_made_where_it_is_read() {
        let dump = build_run_and_read_rc_ir(
            CAPTURES_A_UNIT_SOURCE,
            "max",
            CAPTURES_A_UNIT_OUTPUT,
            "a lambda capturing a unit and reading it",
        );
        assert_closures_at_type_store_nothing(
            &dump,
            "((), Std::I64) -> Std::IO::IOState -> (Std::IO::IOState, ((), Std::I64))",
        );

        // The capture list `Main` declares for the lambda is made where it is read, rather than
        // projected out of a capture object the closure would have had to carry it in.
        assert_a_value_is_made_where_it_is_read(&dump, "Main::#CapList@");
    }

    /// Three lines of output around a call that takes a closure. The `IOState` an `IO` action
    /// threads occupies no storage, so it is among the captures left out of a closure, and what says
    /// it was left out without moving the actions is the order the lines come in.
    const IO_AROUND_A_CLOSURE_SOURCE: &str = r#"
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

    /// What `IO_AROUND_A_CLOSURE_SOURCE` prints: the three lines in the order they are written.
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
                    IO_AROUND_A_CLOSURE_SOURCE,
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
            IO_AROUND_A_CLOSURE_SOURCE,
            "none",
            IO_AROUND_A_CLOSURE_OUTPUT,
            "three `IO` actions around a call taking a closure",
        );
        assert_a_value_is_made_where_it_is_read(&dump, " : Std::IO::IOState ");
    }

    /// A lambda capturing values of both kinds: `u` and `v` occupy no storage, `tag` and `n` occupy
    /// storage, and the body reads all four.
    const MIXED_CAPTURES_SOURCE: &str = r#"
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

    /// What `MIXED_CAPTURES_SOURCE` prints: the two captured values the closure stores, then the
    /// length of the string it measures.
    const MIXED_CAPTURES_OUTPUT: &str = "tag,7,10";

    /// A closure storing some of its captures and leaving the rest out reads every one of them as
    /// the value it was handed: the stored ones are projected at the positions they hold in what the
    /// closure stores, and the rest are made in the function the lambda became.
    ///
    /// The level is `none` for the reason `test_an_iostate_capture_is_left_out_of_the_closure`
    /// gives: at `-O max` a lambda's captures arrive as one capture list, so no closure is handed a
    /// mixture.
    #[test]
    fn test_a_closure_storing_some_of_its_captures_reads_them_all() {
        let dump = build_run_and_read_rc_ir(
            MIXED_CAPTURES_SOURCE,
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

    /// A struct whose only field occupies no storage, acted on over `IO`: `act_u` punches the field
    /// out, and the lambda that plugs it back captures the punched struct, whose type occupies no
    /// storage because the slot the hole keeps does.
    const PUNCHED_CAPTURE_SOURCE: &str = r#"
        module Main;

        type Unitful = unbox struct { u : () };
        type Both = unbox struct { u : (), xs : Array I64 };

        main : IO ();
        main = (
            let s = *Unitful { u : () }.act_u(|u| println("punched").map(|_| u));
            let _ = s;
            let t = *Both { u : (), xs : [10, 20] }.act_u(|u| pure $ u);
            let t = *t.act_xs(|xs| pure $ xs.push_back(30));
            println $ t.@xs.to_iter.map(to_string).join(",")
        );
    "#;

    /// What `PUNCHED_CAPTURE_SOURCE` prints: the line the action writes, then the array the second
    /// struct carries once a value has been plugged back into it.
    const PUNCHED_CAPTURE_OUTPUT: &str = "punched\n10,20,30";

    /// A punched struct that occupies no storage is left out of the closure that plugs a value back
    /// into it, and the value plugged in lands in the struct all the same.
    ///
    /// A punched slot holds no value and keeps the type it was declared at, so whether the struct
    /// occupies storage is decided by every field, the punched one included. `act_` is what hands a
    /// lambda a value at such a type.
    #[test]
    fn test_a_punched_struct_occupying_no_storage_is_plugged_into_all_the_same() {
        for opt_level in ["none", "basic", "max"] {
            assert_eq!(
                build_within_and_run(
                    PUNCHED_CAPTURE_SOURCE,
                    opt_level,
                    Duration::from_secs(600),
                    "`act` on a struct field whose type occupies no storage",
                ),
                PUNCHED_CAPTURE_OUTPUT,
                "the field should be plugged back at -O {}",
                opt_level
            );
        }
    }
    /// Lambdas capturing values that occupy storage, given to the three kinds of built-in that
    /// apply a function in a scope: `borrow_elements` lends a pointer, `mutate_elements` lends one
    /// into a uniquely owned array, and `Destructor::borrow` holds its value retained.
    const SCOPE_BUILTINS_SOURCE: &str = r#"
        module Main;

        main : IO ();
        main = (
            let offset = 3;
            let bytes = "0123456789".get_bytes;
            let length = bytes.borrow_elements(|ptr| FFI_CALL[I64 strlen(Ptr), ptr] + offset);
            let (bytes, _) = bytes.mutate_elements(|ptr|
                FFI_CALL_IO[() memset(Ptr, CInt, CSizeT), ptr, 65.to_CInt, offset.to_CSizeT]
            );
            let dtor = *Destructor::make("n=", |s| pure(s));
            println $ dtor.borrow(|s| s + length.to_string + "," + offset.to_string)
                + "," + bytes.borrow_elements(String::unsafe_from_c_str_ptr)
        );
    "#;

    /// What `SCOPE_BUILTINS_SOURCE` prints.
    const SCOPE_BUILTINS_OUTPUT: &str = "n=13,3,AAA3456789";

    /// The lines of `dump` building a closure that takes its capture list through the environment a
    /// built-in applying a function in a scope hands it. The standard library gives such a built-in
    /// the environment `()`, so the function takes `(((), c), ...)`, where `c` is a capture list
    /// closure specialization minted.
    fn closures_taking_a_capture_list_through_the_environment(dump: &str) -> Vec<&str> {
        dump.lines()
            .filter(|line| {
                line.contains("= closure ")
                    && line.contains(" : (((), ")
                    && line.contains("#CapList@")
            })
            .collect()
    }

    /// A lambda given to a built-in that applies it in a scope takes what it captures through the
    /// environment the built-in hands it, so the closure built from it stores nothing and allocates
    /// no capture object.
    #[test]
    fn test_a_lambda_given_to_a_scope_builtin_stores_nothing() {
        let dump = build_run_and_read_rc_ir(
            SCOPE_BUILTINS_SOURCE,
            "max",
            SCOPE_BUILTINS_OUTPUT,
            "lambdas capturing values, given to built-ins applying a function in a scope",
        );
        let built = closures_taking_a_capture_list_through_the_environment(&dump);
        // The standard library builds closures of this kind for what it prints, so the closure of
        // each built-in's lambda is picked out by its type after the capture list: the rest of the
        // tuple the lambda is applied to, and what the lambda returns.
        for (builtin, type_after_capture_list) in [
            ("borrow_elements", "), Std::Ptr) -> Std::I64 "),
            (
                "mutate_elements",
                "), Std::Ptr, Std::IO::IOState) -> (Std::IO::IOState, ()) ",
            ),
            (
                "Destructor::borrow",
                "), Std::FFI::Destructor (Std::Array Std::U8)) -> Std::Array Std::U8 ",
            ),
        ] {
            assert!(
                built
                    .iter()
                    .any(|line| line.contains(type_after_capture_list)),
                "the lambda given to `{}` should be built as a closure taking its capture list \
                 through the environment:\n{}",
                builtin,
                dump
            );
        }
        for line in built {
            assert!(
                line.trim_end().ends_with("[]"),
                "a closure taking its captures through the environment should store nothing:\n{}",
                line
            );
        }
    }

    /// A function handing the function it takes to a built-in that applies a function in a scope,
    /// inside a lambda that calls it.
    const CALLS_ITS_ARGUMENT_IN_A_SCOPE_SOURCE: &str = r#"
        module Main;

        // Appends up to `max_size` bytes that `write` puts at a pointer, and keeps as many as it
        // reports.
        _append_written : I64 -> (Ptr -> IO I64) -> Array U8 -> Array U8;
        _append_written = |max_size, write, bytes| (
            let size = bytes.@size;
            let bytes = bytes.reserve(size + max_size)._unsafe_grow_size(size + max_size);
            let (bytes, length) = bytes.mutate_elements(|ptr| write(ptr.add_offset(size)));
            bytes.truncate(size + length)
        );

        main : IO ();
        main = (
            let bytes = range(0, 3).fold(Array::empty(0), |i, bytes|
                bytes._append_written(8, |ptr|
                    FFI_CALL_IO[() memset(Ptr, CInt, CSizeT), ptr, (65 + i).to_CInt, 2.to_CSizeT]
                        .map(|_| 2)
                )
            );
            println $ bytes.to_iter.map(to_string).join(",")
        );
    "#;

    /// What `CALLS_ITS_ARGUMENT_IN_A_SCOPE_SOURCE` prints: two bytes from each of the three writes.
    const CALLS_ITS_ARGUMENT_IN_A_SCOPE_OUTPUT: &str = "65,65,66,66,67,67";

    /// A function whose lambda given to a built-in applying a function in a scope calls the function
    /// the caller passed is specialized on that function, as it is where the lambda is given
    /// anywhere else, so no closure is built for what the caller passed.
    #[test]
    fn test_a_function_calling_its_argument_in_a_scope_is_specialized_on_it() {
        let dump = build_run_and_read_rc_ir(
            CALLS_ITS_ARGUMENT_IN_A_SCOPE_SOURCE,
            "max",
            CALLS_ITS_ARGUMENT_IN_A_SCOPE_OUTPUT,
            "a function calling the function it takes inside a built-in applying it in a scope",
        );
        // The lambda `mutate_elements` is given stands as a closure taking its capture list through
        // the environment, which is what says the build reached the scope at all.
        let built = closures_taking_a_capture_list_through_the_environment(&dump);
        assert!(
            built.iter().any(|line| line.contains("Std::Ptr, Std::IO::IOState)")),
            "the lambda given to `mutate_elements` should be built as a closure taking its capture \
             list through the environment:\n{}",
            dump
        );
        // What `main` passes as `write` is a known lambda, so no closure is built for it.
        let write_closures = dump
            .lines()
            .filter(|line| {
                line.contains("= closure ")
                    && line.contains(
                        " : Std::Ptr -> Std::IO::IOState -> (Std::IO::IOState, Std::I64) ",
                    )
            })
            .collect::<Vec<_>>();
        assert!(
            write_closures.is_empty(),
            "no closure should be built for the function passed as `write`:\n{}",
            write_closures.join("\n")
        );
    }

    /// Values captured by lambdas given to built-ins that apply a function in a scope: a lambda
    /// nested in another, whose captures include the pointer the outer one is lent, and a lambda
    /// that captures the array it writes through, so that the write lands on a copy.
    const MOVED_CAPTURES_SOURCE: &str = r#"
        module Main;

        main : IO ();
        main = (
            let offset = 3;
            let bytes = "0123456789".get_bytes;
            let other = "abcdef".get_bytes;
            let nested = bytes.borrow_elements(|p| other.borrow_elements(|q|
                FFI_CALL[I64 strlen(Ptr), p] * 100 + FFI_CALL[I64 strlen(Ptr), q] + offset
            ));
            assert_eq(|_|"nested", nested, 1009);;
            let (written, _) = bytes.mutate_elements(|ptr|
                FFI_CALL_IO[() memset(Ptr, CInt, CSizeT), ptr, bytes.@(1).to_CInt, offset.to_CSizeT]
            );
            assert_eq(|_|"original", bytes.borrow_elements(String::unsafe_from_c_str_ptr), "0123456789");;
            assert_eq(|_|"written", written.borrow_elements(String::unsafe_from_c_str_ptr), "1113456789");;
            let dtor = *Destructor::make("n=", |s| pure(s));
            assert_eq(|_|"retained", dtor.borrow(|s| s + nested.to_string), "n=1009");;
            pure()
        );
    "#;

    /// What the lambdas capture reaches them with the values it had where the lambdas were written,
    /// at every optimization level: through the environment at `-O max`, and through the closure
    /// below it. Memcheck finds no leak or double free in the reference counting of either.
    #[test]
    fn test_captures_moved_into_the_environment_keep_their_values() {
        for opt_level in [
            FixOptimizationLevel::None,
            FixOptimizationLevel::Basic,
            FixOptimizationLevel::Max,
        ] {
            let mut config = Configuration::develop_mode();
            config.set_fix_opt_level(opt_level);
            config.set_valgrind(ValgrindTool::MemCheck);
            test_source(MOVED_CAPTURES_SOURCE, config);
        }
    }
}
