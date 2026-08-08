// Let-elimination replaces `let x = {e0}; {e1}` with `{e1}[x := {e0}]` once it has inspected the
// occurrences of `x` in `{e1}`. Which occurrences belong to that binding is what these tests pin:
// an occurrence under a binder that gives the name to something else belongs to that binder, and an
// occurrence the inspection reaches only through a lambda or a match arm still keeps the binding
// alive. The occurrences are placed in every kind of expression the inspection walks, and in each
// position of a group evaluated together, so that the rewrite keeps the value whether or not
// another local name is read ahead of the binding.

#[cfg(test)]
mod tests {
    use crate::configuration::Configuration;
    use crate::tests::test_util::{build_within_and_run, test_source};
    use std::time::Duration;

    /// The only occurrence of `x` is inside a lambda, which captures the binding.
    #[test]
    fn test_name_used_only_inside_a_lambda_keeps_its_binding() {
        let source = r#"
        module Main;

        used_in_lambda : I64 -> I64;
        used_in_lambda = |a| (
            let x = a + 1;
            let f = |v : I64| v + x;
            f(10)
        );

        main : IO ();
        main = (
            assert_eq(|_|"the lambda adds what `x` is bound to", used_in_lambda(5), 16);;
            pure()
        );
        "#;
        test_source(source, Configuration::develop_mode());
    }

    /// The only occurrence of `x` is inside the value expression of a match arm.
    #[test]
    fn test_name_used_only_inside_a_match_arm_keeps_its_binding() {
        let source = r#"
        module Main;

        used_in_match_arm : I64 -> I64;
        used_in_match_arm = |a| (
            let x = a + 1;
            match Option::some(3) {
                some(v) => v + x,
                none(_) => 0
            }
        );

        main : IO ();
        main = (
            assert_eq(|_|"the arm adds what `x` is bound to", used_in_match_arm(5), 9);;
            pure()
        );
        "#;
        test_source(source, Configuration::develop_mode());
    }

    /// A lambda parameter of the same name gives `x` inside the lambda to the parameter.
    #[test]
    fn test_name_rebound_by_a_lambda_parameter_denotes_the_parameter() {
        let source = r#"
        module Main;

        rebound_by_lambda_param : I64 -> I64;
        rebound_by_lambda_param = |a| (
            let x = a + 1;
            let g = |x : I64| x * 2;
            g(10) + x
        );

        main : IO ();
        main = (
            assert_eq(|_|"`x` in the lambda is its parameter", rebound_by_lambda_param(5), 26);;
            pure()
        );
        "#;
        test_source(source, Configuration::develop_mode());
    }

    /// An inner `let` of the same name gives `x` in its value expression to the inner binding.
    #[test]
    fn test_name_rebound_by_an_inner_let_denotes_the_inner_binding() {
        let source = r#"
        module Main;

        rebound_by_inner_let : I64 -> I64;
        rebound_by_inner_let = |a| (
            let x = a + 1;
            let y = (let x = 100; x * 2);
            y + x
        );

        main : IO ();
        main = (
            assert_eq(|_|"`x` in the inner `let` is what that `let` binds", rebound_by_inner_let(5), 206);;
            pure()
        );
        "#;
        test_source(source, Configuration::develop_mode());
    }

    /// The bound expression of a `let` is evaluated in the enclosing scope, so it reaches the outer
    /// binding even when the pattern gives the same name to what it binds.
    #[test]
    fn test_name_read_by_the_bound_expression_of_a_rebinding_let_keeps_its_binding() {
        let source = r#"
        module Main;

        rebinding_let_reads_the_outer_name : I64 -> I64;
        rebinding_let_reads_the_outer_name = |a| (
            let x = a + 1;
            let x = x * 2;
            x + x
        );

        main : IO ();
        main = (
            assert_eq(|_|"the bound expression doubles what the outer `x` is bound to", rebinding_let_reads_the_outer_name(5), 24);;
            pure()
        );
        "#;
        test_source(source, Configuration::develop_mode());
    }

    /// The expression a match inspects is evaluated in the enclosing scope, so it reaches the outer
    /// binding even when every arm gives the same name to what its pattern binds.
    #[test]
    fn test_name_read_by_a_match_condition_keeps_its_binding() {
        let source = r#"
        module Main;

        match_condition_reads_the_outer_name : I64 -> I64;
        match_condition_reads_the_outer_name = |a| (
            let x = a + 1;
            match Option::some(x) {
                some(x) => x * 3,
                none(_) => 0
            }
        );

        main : IO ();
        main = (
            assert_eq(|_|"the match inspects what the outer `x` is bound to", match_condition_reads_the_outer_name(5), 18);;
            pure()
        );
        "#;
        test_source(source, Configuration::develop_mode());
    }

    /// A match pattern of the same name gives `x` in that arm to the pattern, while the arm whose
    /// pattern binds another name reaches the outer binding.
    #[test]
    fn test_name_rebound_by_a_match_pattern_denotes_the_pattern() {
        let source = r#"
        module Main;

        rebound_by_match_pattern : I64 -> I64;
        rebound_by_match_pattern = |a| (
            let x = a + 1;
            match Option::some(7) {
                some(x) => x * 3,
                none(_) => x
            }
        );

        main : IO ();
        main = (
            assert_eq(|_|"`x` in the arm is what the pattern binds", rebound_by_match_pattern(5), 21);;
            pure()
        );
        "#;
        test_source(source, Configuration::develop_mode());
    }

    /// A destructuring `let` pattern gives `x` after it to what the pattern binds, while its bound
    /// expression, evaluated in the enclosing scope, still reads the outer binding.
    #[test]
    fn test_name_rebound_by_a_destructuring_let_pattern_denotes_the_pattern() {
        let source = r#"
        module Main;

        type Pair = unbox struct { fst : I64, snd : I64 };

        rebound_by_destructuring_pattern : I64 -> I64;
        rebound_by_destructuring_pattern = |a| (
            let x = a + 1;
            let Pair { fst : x, snd : y } = Pair { fst : x * 10, snd : 7 };
            x + y
        );

        main : IO ();
        main = (
            assert_eq(|_|"`x` after the pattern is what the pattern binds", rebound_by_destructuring_pattern(5), 67);;
            pure()
        );
        "#;
        test_source(source, Configuration::develop_mode());
    }

    /// A name several levels down a nested pattern binds just as one at the top of it does.
    #[test]
    fn test_name_rebound_deep_inside_a_nested_pattern_denotes_the_pattern() {
        let source = r#"
        module Main;

        type Pair = unbox struct { fst : I64, snd : I64 };
        type Nest = unbox struct { inner : Pair, tag : I64 };

        rebound_deep_in_pattern : I64 -> I64;
        rebound_deep_in_pattern = |a| (
            let x = a + 1;
            let Nest { inner : Pair { fst : x, snd : y }, tag : t } =
                Nest { inner : Pair { fst : x * 10, snd : 7 }, tag : x };
            x + y + t
        );

        main : IO ();
        main = (
            assert_eq(|_|"`x` after the pattern is what the pattern binds", rebound_deep_in_pattern(5), 73);;
            pure()
        );
        "#;
        test_source(source, Configuration::develop_mode());
    }

    /// A union pattern of a match arm gives `x` in that arm to what it binds, while an arm whose
    /// pattern binds another name reaches the outer binding.
    #[test]
    fn test_name_rebound_by_a_union_pattern_denotes_the_pattern() {
        let source = r#"
        module Main;

        type Pair = unbox struct { fst : I64, snd : I64 };
        type Choice = unbox union { one : I64, two : Pair };

        rebound_by_union_pattern : I64 -> I64;
        rebound_by_union_pattern = |a| (
            let x = a + 1;
            let inner = match Choice::two(Pair { fst : 3, snd : 4 }) {
                one(v) => v + x,
                two(Pair { fst : x, snd : s }) => x * 1000 + s
            };
            let outer = match Choice::one(5) {
                one(v) => v + x,
                two(p) => Pair::@fst(p)
            };
            inner + outer
        );

        main : IO ();
        main = (
            assert_eq(|_|"each arm reads the binding its own pattern leaves standing", rebound_by_union_pattern(5), 3015);;
            pure()
        );
        "#;
        test_source(source, Configuration::develop_mode());
    }

    /// A chain of `let`s that each give the same name to a new binding, each bound expression
    /// reading the binding before it.
    #[test]
    fn test_a_chain_of_rebindings_reads_the_binding_before_each() {
        let source = r#"
        module Main;

        chain_of_rebindings : I64 -> I64;
        chain_of_rebindings = |a| (
            let x = a + 1;
            let x = (
                let x = (
                    let x = x * 2;
                    x + 1
                );
                x * 3
            );
            x + 1
        );

        main : IO ();
        main = (
            assert_eq(|_|"each `let` reads the binding before it", chain_of_rebindings(5), 40);;
            pure()
        );
        "#;
        test_source(source, Configuration::develop_mode());
    }

    /// The only occurrence of `x` is under a type annotation.
    #[test]
    fn test_name_used_only_under_a_type_annotation_keeps_its_binding() {
        let source = r#"
        module Main;

        used_under_type_annotation : I64 -> I64;
        used_under_type_annotation = |a| (
            let x = a + 1;
            (x : I64) * 10
        );

        main : IO ();
        main = (
            assert_eq(|_|"the annotated expression is what `x` is bound to", used_under_type_annotation(5), 60);;
            pure()
        );
        "#;
        test_source(source, Configuration::develop_mode());
    }

    /// The only occurrence of `x` is an argument to an inline-LLVM expression, which an array read
    /// compiles to.
    #[test]
    fn test_name_used_only_as_an_llvm_argument_keeps_its_binding() {
        let source = r#"
        module Main;

        used_as_llvm_argument : I64 -> I64;
        used_as_llvm_argument = |a| (
            let x = [a, a + 1, a + 2];
            x.@(2)
        );

        main : IO ();
        main = (
            assert_eq(|_|"the read answers the last element of what `x` is bound to", used_as_llvm_argument(5), 7);;
            pure()
        );
        "#;
        test_source(source, Configuration::develop_mode());
    }

    /// The only occurrence of `x` is the function of an application, and `x` is bound to a strictly
    /// partial application of names to a global lambda.
    #[test]
    fn test_name_bound_to_a_partial_application_and_applied_once() {
        let source = r#"
        module Main;

        add3 : I64 -> I64 -> I64 -> I64;
        add3 = |a, b, c| a + b + c;

        applied_partial_application : I64 -> I64;
        applied_partial_application = |a| (
            let g = add3(a);
            g(20, 300)
        );

        main : IO ();
        main = (
            assert_eq(|_|"the call sums the parameter and the two arguments", applied_partial_application(5), 325);;
            pure()
        );
        "#;
        test_source(source, Configuration::develop_mode());
    }

    /// The binding is read in a later field of a struct expression while an earlier field reads a
    /// name of the enclosing scope.
    #[test]
    fn test_binding_read_in_a_later_struct_field_keeps_its_value() {
        let source = r#"
        module Main;

        type Pair = struct { a : I64, b : I64 };

        read_in_later_field : Array I64 -> I64;
        read_in_later_field = |arr| (
            let x = arr.get_size * 10;
            let p = Pair { a : arr.@(0), b : x };
            p.@a + p.@b
        );

        main : IO ();
        main = (
            assert_eq(|_|"the fields hold the first element and ten times the size", read_in_later_field([7, 8, 9]), 37);;
            pure()
        );
        "#;
        test_source(source, Configuration::develop_mode());
    }

    /// The binding is read in a middle element of an array literal whose other elements read a name
    /// of the enclosing scope.
    #[test]
    fn test_binding_read_in_an_array_literal_element_keeps_its_value() {
        let source = r#"
        module Main;

        read_in_element : Array I64 -> I64;
        read_in_element = |arr| (
            let x = arr.get_size * 10;
            let lit = [arr.@(0), x, arr.@(1)];
            lit.@(0) + lit.@(1) + lit.@(2)
        );

        main : IO ();
        main = (
            assert_eq(|_|"the literal holds two elements and ten times the size", read_in_element([7, 8, 9]), 45);;
            pure()
        );
        "#;
        test_source(source, Configuration::develop_mode());
    }

    /// The binding is read in a match arm whose condition reads a name of the enclosing scope.
    #[test]
    fn test_binding_read_in_an_arm_of_a_match_on_an_enclosing_name_keeps_its_value() {
        let source = r#"
        module Main;

        read_in_arm : Array I64 -> I64;
        read_in_arm = |arr| (
            let x = arr.get_size * 10;
            match Option::some(arr.@(0)) {
                some(v) => v + x,
                none(_) => x
            }
        );

        main : IO ();
        main = (
            assert_eq(|_|"the arm adds the first element to ten times the size", read_in_arm([7, 8, 9]), 37);;
            pure()
        );
        "#;
        test_source(source, Configuration::develop_mode());
    }

    /// The binding is read in a branch of an `if` whose condition reads a name of the enclosing
    /// scope.
    #[test]
    fn test_binding_read_in_a_branch_of_an_if_on_an_enclosing_name_keeps_its_value() {
        let source = r#"
        module Main;

        read_in_branch : Array I64 -> I64;
        read_in_branch = |arr| (
            let x = arr.get_size * 10;
            if arr.@(0) > 0 { x + arr.@(1) } else { 0 }
        );

        main : IO ();
        main = (
            assert_eq(|_|"the branch adds the second element to ten times the size", read_in_branch([7, 8, 9]), 38);;
            pure()
        );
        "#;
        test_source(source, Configuration::develop_mode());
    }

    /// The binding is read in the main expression of an `eval` whose side expression reads a name of
    /// the enclosing scope, and in the body of a lambda that captures such a name too.
    #[test]
    fn test_binding_read_after_an_eval_and_inside_a_capturing_lambda_keeps_its_value() {
        let source = r#"
        module Main;

        read_after_eval : Array I64 -> I64;
        read_after_eval = |arr| (
            let x = arr.get_size * 10;
            eval arr.@(0);
            x + 1
        );

        read_inside_lambda : Array I64 -> I64;
        read_inside_lambda = |arr| (
            let x = arr.get_size * 10;
            let h = |k : I64| k + x + arr.@(0);
            h(1) + h(2)
        );

        main : IO ();
        main = (
            assert_eq(|_|"the eval leaves ten times the size plus one", read_after_eval([7, 8, 9]), 31);;
            assert_eq(|_|"the lambda adds what `x` is bound to on each call", read_inside_lambda([7, 8, 9]), 77);;
            pure()
        );
        "#;
        test_source(source, Configuration::develop_mode());
    }

    /// The binding is read in an argument of an FFI call whose other argument reads a name of the
    /// enclosing scope.
    #[test]
    fn test_binding_read_in_an_ffi_call_argument_keeps_its_value() {
        let source = r#"
        module Main;

        read_in_ffi_argument : Array U8 -> I64;
        read_in_ffi_argument = |bytes| (
            let n = bytes.get_size.to_I64;
            let c = bytes.@(0).to_I64;
            let a = FFI_CALL[CInt abs(CInt), (0 - c).to_CInt];
            a.to_I64 + n
        );

        main : IO ();
        main = (
            assert_eq(|_|"the call returns the magnitude of the first byte plus the size", read_in_ffi_argument([65_U8, 66_U8, 67_U8]), 68);;
            pure()
        );
        "#;
        test_source(source, Configuration::develop_mode());
    }

    /// The binding is read in the second argument of a dot-form call whose receiver is the array the
    /// bound expression reads.
    #[test]
    fn test_binding_read_beside_its_own_array_keeps_its_value() {
        let source = r#"
        module Main;

        set_from_size : Array I64 -> Array I64;
        set_from_size = |arr| (
            let x = arr.get_size + 100;
            arr.set(0, x)
        );

        set_from_element : Array I64 -> Array I64;
        set_from_element = |arr| (
            let y = arr.@(1) * 1000;
            arr.set(2, y)
        );

        digits : Array I64 -> I64;
        digits = |arr| arr.to_iter.fold(0, |e, s| s * 10 + e);

        main : IO ();
        main = (
            assert_eq(|_|"the first element becomes the size plus a hundred", set_from_size([1, 2, 3]).digits, 10323);;
            assert_eq(|_|"the last element becomes a thousand times the second", set_from_element([1, 2, 3]).digits, 2120);;
            pure()
        );
        "#;
        test_source(source, Configuration::develop_mode());
    }

    // A chain this long builds in a few seconds while the inspection of a binding is proportional
    // to where its name is read, and in several minutes while it walks the whole body of every
    // binding.
    const CHAIN_LENGTH: usize = 2400;

    // Generous next to the few seconds the build takes, with room for a machine several times
    // slower running the rest of the suite beside it, and well short of the minutes the build takes
    // once the inspection walks whole bodies again.
    const TIMEOUT: Duration = Duration::from_secs(120);

    /// Builds and runs a global whose body is a chain of `CHAIN_LENGTH` `let`s, failing if the
    /// build does not finish within `TIMEOUT`. `-O max` is the level that inspects the chain twice:
    /// once for the local inlining that eliminates `let`s, and again for the eta expansion that
    /// uncurrying runs the same pass on.
    ///
    /// Each binding calls a global of one parameter rather than writing the arithmetic out, which
    /// keeps type checking a small part of the budget: an operator resolved through a trait carries
    /// a predicate per occurrence, and a chain of those spends most of the build before the
    /// inspection under test even runs.
    #[test]
    fn test_long_let_chain_compiles_in_reasonable_time() {
        let mut body = String::from("    let a0 = succ(x);\n");
        for i in 1..CHAIN_LENGTH {
            body.push_str(&format!("    let a{} = succ(a{});\n", i, i - 1));
        }
        body.push_str(&format!("    a{}\n", CHAIN_LENGTH - 1));
        let source = format!(
            "module Main;\n\
             \n\
             succ : I64 -> I64;\n\
             succ = |x| x + 1;\n\
             \n\
             let_chain : I64 -> I64;\n\
             let_chain = |x| (\n{body});\n\
             \n\
             main : IO ();\n\
             main = println(let_chain(0).to_string);\n"
        );

        let printed = build_within_and_run(
            &source,
            "max",
            TIMEOUT,
            &format!("a chain of {} `let`s", CHAIN_LENGTH),
        );
        assert_eq!(
            printed,
            CHAIN_LENGTH.to_string(),
            "the chain of {} `let`s returned a wrong value",
            CHAIN_LENGTH
        );
    }
}
