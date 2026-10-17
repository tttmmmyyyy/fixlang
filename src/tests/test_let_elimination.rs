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

    /// The only occurrence of `x` is an argument to a builtin expression, which an array read
    /// compiles to.
    #[test]
    fn test_name_used_only_as_a_builtin_argument_keeps_its_binding() {
        let source = r#"
        module Main;

        used_as_builtin_argument : I64 -> I64;
        used_as_builtin_argument = |a| (
            let x = [a, a + 1, a + 2];
            x.@(2)
        );

        main : IO ();
        main = (
            assert_eq(|_|"the read answers the last element of what `x` is bound to", used_as_builtin_argument(5), 7);;
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

    /// `partial` is bound to a strictly partial application and eliminated, which leaves `reader`
    /// bound to a chain of two applications that together hand `make_reader` every parameter it
    /// takes. `reader` is applied once, after the program asks whether the cell the call reads is
    /// unique. The chain is not strictly partial, so its binding stays, and the call has released
    /// the cell by the time the program asks.
    #[test]
    fn test_name_bound_to_a_saturated_application_chain_keeps_its_binding() {
        let source = r#"
        module Main;

        type Cell = box struct { v : I64 };

        make_reader : I64 -> Cell -> I64 -> I64 -> I64;
        make_reader = |d, cell, k| (
            if d > 0 { make_reader(d - 1, cell, k) };
            let v = cell.@v;
            |x| x + v + k
        );

        main : IO ();
        main = (
            let k = (*get_args).@size;
            let cell = Cell { v : 10 };
            let partial = make_reader(k);
            let reader = partial(cell, k);
            let (unique, cell_after) = cell.unsafe_is_unique;
            assert_eq(|_|"the reader adds the value and `k` to its argument", reader(cell_after.@v), 21);;
            assert_eq(|_|"the call has released the cell before the program asks", unique, true);;
            pure()
        );
        "#;
        test_source(source, Configuration::develop_mode());
    }

    /// `v` reads the cell, and is read in the last argument of a call whose earlier argument hands
    /// the cell to `is_unique_after`. Both functions call themselves, so they keep their call sites,
    /// and the call takes all its arguments at once after uncurrying. The arguments are evaluated
    /// from left to right, so the binding stays ahead of the call, the cell's last use is the call
    /// of `is_unique_after`, and the cell is unique there.
    #[test]
    fn test_name_read_in_a_later_argument_of_a_call_keeps_its_binding() {
        let source = r#"
        module Main;

        type Cell = box struct { v : I64 };

        is_unique_after : I64 -> Cell -> Bool;
        is_unique_after = |n, cell| if n <= 0 { cell.unsafe_is_unique.@0 } else { is_unique_after(n - 1, cell) };

        show_both : I64 -> Bool -> I64 -> String;
        show_both = |n, unique, v| if n <= 0 { unique.to_string + " " + v.to_string } else { show_both(n - 1, unique, v) };

        main : IO ();
        main = (
            let k = (*get_args).@size;
            let cell = Cell { v : 10 + k };
            let v = cell.@v;
            assert_eq(|_|"the cell is unique where the call reads it", show_both(k, is_unique_after(k, cell), v), "true 11");;
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

    /// `r` reads the cell, and is read in the function of a dot-form call whose receiver hands the
    /// cell to `unsafe_is_unique`. The binding stays ahead of the receiver, so the cell is unique
    /// where the receiver reads it.
    #[test]
    fn test_binding_read_in_the_function_of_a_dot_call_stays_ahead_of_the_receiver() {
        let source = r#"
        module Main;

        type Cell = box struct { v : I64 };

        pair_with : I64 -> (Bool, Cell) -> (Bool, I64);
        pair_with = |r, p| (p.@0, r + p.@1.@v);

        main : IO ();
        main = (
            let k = (*get_args).@size;
            let cell = Cell { v : k };
            let r = cell.@v;
            assert_eq(|_|"the receiver reads the cell while it is unique", cell.unsafe_is_unique.pair_with(r), (true, 2));;
            pure()
        );
        "#;
        test_source(source, Configuration::develop_mode());
    }

    /// `f` is bound to an `if` that reads the cell, and is applied to an argument that hands the
    /// cell to `unsafe_is_unique`. The binding stays ahead of the argument, so the cell is unique
    /// where the argument reads it.
    #[test]
    fn test_name_of_an_if_applied_to_an_argument_stays_ahead_of_the_argument() {
        let source = r#"
        module Main;

        type Cell = box struct { v : I64 };

        pair_with : I64 -> (Bool, Cell) -> (Bool, I64);
        pair_with = |r, p| (p.@0, r + p.@1.@v);

        main : IO ();
        main = (
            let k = (*get_args).@size;
            let cell = Cell { v : k };
            let f = if cell.@v > 0 { pair_with(1) } else { pair_with(2) };
            assert_eq(|_|"the argument reads the cell while it is unique", f(cell.unsafe_is_unique), (true, 2));;
            pure()
        );
        "#;
        test_source(source, Configuration::develop_mode());
    }

    /// `r` reads the cell, and is read in the condition of an `if` that is applied first to a
    /// number and then to an argument that hands the cell to `unsafe_is_unique`. The binding stays
    /// ahead of the argument, so the cell is unique where the argument reads it.
    #[test]
    fn test_binding_read_in_an_if_on_the_spine_of_a_call_stays_ahead_of_the_last_argument() {
        let source = r#"
        module Main;

        type Cell = box struct { v : I64 };

        pair_with : I64 -> (Bool, Cell) -> (Bool, I64);
        pair_with = |r, p| (p.@0, r + p.@1.@v);

        pair_with_negated : I64 -> (Bool, Cell) -> (Bool, I64);
        pair_with_negated = |r, p| (p.@0, p.@1.@v - r);

        main : IO ();
        main = (
            let k = (*get_args).@size;
            let cell = Cell { v : k };
            let r = cell.@v;
            assert_eq(|_|"the argument reads the cell while it is unique", (if r > 0 { pair_with } else { pair_with_negated })(k + 1)(cell.unsafe_is_unique), (true, 3));;
            pure()
        );
        "#;
        test_source(source, Configuration::develop_mode());
    }

    /// `r` reads the cell, and is read in the side expression of an `eval` applied to an argument
    /// that hands the cell to `unsafe_is_unique`. The binding stays ahead of the argument, so the
    /// cell is unique where the argument reads it.
    #[test]
    fn test_binding_read_in_an_eval_applied_to_an_argument_stays_ahead_of_the_argument() {
        let source = r#"
        module Main;

        type Cell = box struct { v : I64 };

        pair_with : I64 -> (Bool, Cell) -> (Bool, I64);
        pair_with = |r, p| (p.@0, r + p.@1.@v);

        main : IO ();
        main = (
            let k = (*get_args).@size;
            let cell = Cell { v : k };
            let r = cell.@v;
            assert_eq(|_|"the argument reads the cell while it is unique", (eval assert(|_|"r is positive", r > 0); pair_with(1))(cell.unsafe_is_unique), (true, 2));;
            pure()
        );
        "#;
        test_source(source, Configuration::develop_mode());
    }

    /// `r` reads the cell, and is read in the condition of a `match` applied to an argument that
    /// hands the cell to `unsafe_is_unique`. The binding stays ahead of the argument, so the cell is
    /// unique where the argument reads it.
    #[test]
    fn test_binding_read_in_a_match_applied_to_an_argument_stays_ahead_of_the_argument() {
        let source = r#"
        module Main;

        type Cell = box struct { v : I64 };

        pair_with : I64 -> (Bool, Cell) -> (Bool, I64);
        pair_with = |r, p| (p.@0, r + p.@1.@v);

        main : IO ();
        main = (
            let k = (*get_args).@size;
            let cell = Cell { v : k };
            let r = cell.@v;
            assert_eq(|_|"the argument reads the cell while it is unique", (match (r, 3) { (a, b) => if a > b { pair_with(a) } else { pair_with(b + 1) } })(cell.unsafe_is_unique), (true, 5));;
            pure()
        );
        "#;
        test_source(source, Configuration::develop_mode());
    }

    /// `x` reads the cell, and is read in the bound expression of a `let` applied to an argument
    /// that hands the cell to `unsafe_is_unique`; the value of that `let` is an `if`. The call of
    /// `fix` makes `defunctionalize_fix` run let elimination on `main`, which eliminates the `let`
    /// in the function position and leaves the argument to be evaluated ahead of the `if`.
    /// The binding of `x` stays ahead of the argument, so the cell is unique where the argument
    /// reads it.
    #[test]
    fn test_binding_read_in_the_bound_expression_of_a_let_applied_to_an_argument_stays_ahead_of_the_argument(
    ) {
        let source = r#"
        module Main;

        type Cell = box struct { v : I64 };

        pair_with : I64 -> (Bool, Cell) -> (Bool, I64);
        pair_with = |r, p| (p.@0, r + p.@1.@v);

        flag : Bool;
        flag = FFI_CALL[I32 getpid()] > 0.i32;

        main : IO ();
        main = (
            let k = (*get_args).@size;
            let cell = Cell { v : k };
            let c = flag;
            let x = cell.@v;
            let res = (let pair_with_x = pair_with(x) in if c { pair_with_x } else { pair_with(0) })(cell.unsafe_is_unique);
            let d = fix(|f, n| if n == 0 { 0 } else { f(n - 1) }, k);
            assert_eq(|_|"the argument reads the cell while it is unique", res, (true, k + k));;
            assert_eq(|_|"the recursion counts down to zero", d, 0);;
            pure()
        );
        "#;
        test_source(source, Configuration::develop_mode());
    }

    /// A chain this long builds in a few seconds while the inspection of a binding is proportional
    /// to where its name is read, and in several minutes while it walks the whole body of every
    /// binding.
    const CHAIN_LENGTH: usize = 2400;

    /// Generous next to the few seconds the build takes, with room for a machine several times
    /// slower running the rest of the suite beside it, and well short of the minutes the build
    /// takes once the inspection walks whole bodies.
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
