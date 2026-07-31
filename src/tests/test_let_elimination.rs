// Let-elimination replaces `let x = {e0}; {e1}` with `{e1}[x := {e0}]` once it has inspected the
// occurrences of `x` in `{e1}`. Which occurrences belong to that binding is what these tests pin:
// an occurrence under a binder that gives the name to something else belongs to that binder, and an
// occurrence the inspection reaches only through a lambda or a match arm still keeps the binding
// alive.

#[cfg(test)]
mod tests {
    use crate::configuration::Configuration;
    use crate::tests::test_util::test_source;

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
}
