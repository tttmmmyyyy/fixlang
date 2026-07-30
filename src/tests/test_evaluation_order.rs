// A call whose arguments can each stop the program -- through `Debug::assert` or `undefined` --
// makes the order they are evaluated in observable: the argument evaluated first is the one that
// stops it. Each test below pins one call shape at whatever optimization level the suite is given.
// The shapes are the ones the compiler orders by how the call is written; the order of other calls
// is left unspecified.

#[cfg(test)]
mod tests {
    use crate::configuration::Configuration;
    use crate::tests::test_util::test_source_fail;

    /// A call of two arguments evaluates the one written first.
    #[test]
    fn test_prefix_call_evaluates_the_argument_written_first() {
        let source = r#"
        module Main;

        add_two : I64 -> I64 -> I64;
        add_two = |a, b| a + b;

        main : IO ();
        main = println $ add_two(undefined("first argument"), undefined("second argument")).to_string;
        "#;
        test_source_fail(source, Configuration::develop_mode(), "first argument");
    }

    /// A call of three arguments, to a function that is called from itself and so keeps its call
    /// sites, evaluates the argument written first.
    #[test]
    fn test_call_of_three_arguments_evaluates_the_argument_written_first() {
        let source = r#"
        module Main;

        add_three_counting_down : I64 -> I64 -> I64 -> I64;
        add_three_counting_down = |a, b, c| if a <= 0 { b + c } else { add_three_counting_down(a - 1, b, c) };

        main : IO ();
        main = println $ add_three_counting_down(undefined("first"), undefined("second"), undefined("third")).to_string;
        "#;
        test_source_fail(source, Configuration::develop_mode(), "first");
    }

    /// A binary operator evaluates its left operand, the one written first.
    #[test]
    fn test_operator_evaluates_the_operand_written_first() {
        let source = r#"
        module Main;

        after_countdown : I64 -> I64;
        after_countdown = |n| if n <= 0 { undefined("first operand") + undefined("second operand") } else { after_countdown(n - 1) };

        main : IO ();
        main = println $ after_countdown(1).to_string;
        "#;
        test_source_fail(source, Configuration::develop_mode(), "first operand");
    }

    /// Where the first argument of a call of three arguments cannot stop the program, the second is
    /// evaluated before the third.
    #[test]
    fn test_call_of_three_arguments_evaluates_the_second_before_the_third() {
        let source = r#"
        module Main;

        add_three_counting_down : I64 -> I64 -> I64 -> I64;
        add_three_counting_down = |a, b, c| if a <= 0 { b + c } else { add_three_counting_down(a - 1, b, c) };

        main : IO ();
        main = println $ add_three_counting_down(0, undefined("second argument"), undefined("third argument")).to_string;
        "#;
        test_source_fail(source, Configuration::develop_mode(), "second argument");
    }

    /// A call whose arguments are themselves calls evaluates the argument written first.
    #[test]
    fn test_call_with_computed_arguments_evaluates_the_argument_written_first() {
        let source = r#"
        module Main;

        add_zero : I64 -> I64;
        add_zero = |x| x + 0;

        add_two : I64 -> I64 -> I64;
        add_two = |a, b| if a <= -1 { b } else { a + b };

        after_countdown : I64 -> I64;
        after_countdown = |n| if n <= 0 {
            add_two(add_zero(undefined("first argument")), add_zero(undefined("second argument")))
        } else { after_countdown(n - 1) };

        main : IO ();
        main = println $ after_countdown(1).to_string;
        "#;
        test_source_fail(source, Configuration::develop_mode(), "first argument");
    }

    /// The `>>` composition operator evaluates the operand written first.
    #[test]
    fn test_forward_composition_evaluates_the_operand_written_first() {
        let source = r#"
        module Main;

        after_countdown : I64 -> I64;
        after_countdown = |n| if n <= 0 {
            ((undefined("first operand") : I64 -> I64) >> (undefined("second operand") : I64 -> I64))(0)
        } else { after_countdown(n - 1) };

        main : IO ();
        main = println $ after_countdown(1).to_string;
        "#;
        test_source_fail(source, Configuration::develop_mode(), "first operand");
    }
}
