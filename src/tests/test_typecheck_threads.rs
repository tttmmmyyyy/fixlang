// The type checker checks the global values of a program on the compiler's worker threads. The
// errors it reports come out in one order however the threads interleave.

#[cfg(test)]
mod tests {
    use crate::configuration::{Configuration, SubCommand};
    use crate::elaboration::elaborate_via_config;
    use crate::error::panic_if_err;
    use crate::misc::save_temporary_source;

    /// The errors `source` fails to type-check with, as the compiler renders them, with the values
    /// checked on `num_threads` threads.
    fn type_errors(source: &str, num_threads: usize) -> String {
        let mut config = panic_if_err(Configuration::release_mode(SubCommand::Build));
        config.num_worker_thread = num_threads;
        let saved_source = panic_if_err(save_temporary_source(source, "typecheck_threads"));
        config.add_user_source_file(saved_source.file_path);
        match elaborate_via_config(&config) {
            Ok(_) => panic!("the program type-checks"),
            Err(errors) => errors.to_string(),
        }
    }

    /// Type-checking on several threads reports the errors of a program in the order one thread
    /// reports them.
    #[test]
    fn test_type_errors_on_worker_threads_come_in_the_order_one_thread_gives() {
        let mut source = String::from("module Main;\n");
        let value_count = 16;
        for i in 0..value_count {
            source += &format!("value{} : I64;\nvalue{} = \"not a number {}\";\n", i, i, i);
        }
        let sum = (0..value_count)
            .map(|i| format!("value{}", i))
            .collect::<Vec<_>>()
            .join(" + ");
        source += &format!("main : IO ();\nmain = println(({}).to_string);\n", sum);

        let errors = type_errors(&source, 1);
        assert_eq!(
            errors.matches("not a number").count(),
            value_count,
            "every value is reported:\n{}",
            errors
        );
        for num_threads in [2, 4, 8, 16] {
            assert_eq!(
                errors,
                type_errors(&source, num_threads),
                "the errors found on {} threads come in another order",
                num_threads
            );
        }
    }
}
