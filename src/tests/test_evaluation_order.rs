// A call whose arguments can each stop the program -- through `Debug::assert` or `undefined` --
// makes the order they are evaluated in observable: the argument evaluated first is the one that
// stops it. Each test below pins one call shape at whatever optimization level the suite is given.
// The shapes are ones the compiler orders by how the call is written; it does not order every call
// that way.

#[cfg(test)]
mod tests {
    use crate::configuration::Configuration;
    use crate::tests::test_util::test_source_fail;

    /// A call of two arguments evaluates the one written first.
    #[test]
    fn test_prefix_call_evaluates_the_argument_written_first() {
        let source = r#"
        module Main;

        pair_up : I64 -> I64 -> I64;
        pair_up = |a, b| a + b;

        main : IO ();
        main = println $ pair_up(undefined("first argument"), undefined("second argument")).to_string;
        "#;
        test_source_fail(source, Configuration::develop_mode(), "first argument");
    }

    /// A call of three arguments, to a function that is called from itself and so keeps its call
    /// sites, evaluates the argument written first.
    #[test]
    fn test_call_of_three_arguments_evaluates_the_argument_written_first() {
        let source = r#"
        module Main;

        triple : I64 -> I64 -> I64 -> I64;
        triple = |a, b, c| if a <= 0 { b + c } else { triple(a - 1, b, c) };

        main : IO ();
        main = println $ triple(undefined("first"), undefined("second"), undefined("third")).to_string;
        "#;
        test_source_fail(source, Configuration::develop_mode(), "first");
    }

    /// A binary operator evaluates its left operand, the one written first.
    #[test]
    fn test_operator_evaluates_the_operand_written_first() {
        let source = r#"
        module Main;

        pick : I64 -> I64;
        pick = |n| if n <= 0 { undefined("first operand") + undefined("second operand") } else { pick(n - 1) };

        main : IO ();
        main = println $ pick(1).to_string;
        "#;
        test_source_fail(source, Configuration::develop_mode(), "first operand");
    }
}
