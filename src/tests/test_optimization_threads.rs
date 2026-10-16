// The optimization passes rewrite the symbols of a program on the compiler's worker threads. The
// program they leave has to be the one a single thread leaves: the same symbols, defined the same
// way, held in the same order, since the stages after them generate code in that order.

#[cfg(test)]
mod tests {
    use crate::configuration::{Configuration, FixOptimizationLevel, SubCommand};
    use crate::elaboration::elaborate_via_config;
    use crate::error::panic_if_err;
    use crate::misc::save_temporary_source;
    use crate::optimization::optimization;

    /// The names of the symbols `source` is optimized into, in the order the program holds them, and
    /// the text of their definitions, with the passes running on `num_threads` threads.
    fn optimized_symbols(source: &str, num_threads: usize) -> (Vec<String>, String) {
        let mut config = panic_if_err(Configuration::release_mode(SubCommand::Build));
        config.set_fix_opt_level(FixOptimizationLevel::Max);
        config.num_worker_thread = num_threads;
        let saved_source = panic_if_err(save_temporary_source(source, "optimization_threads"));
        config.add_user_source_file(saved_source.file_path);
        let mut program = panic_if_err(elaborate_via_config(&config));
        optimization::run(&mut program, &config);
        let names = program
            .symbols
            .keys()
            .map(|name| name.to_string())
            .collect();
        (names, program.stringify_symbols().to_string())
    }

    /// Optimizing on several threads leaves the same program as optimizing on one thread, for a
    /// program that every pass running on the worker threads rewrites: globals to inline, struct
    /// arguments to split, constructions to read, closures to specialize and lift, and functions to
    /// uncurry.
    ///
    /// A pass whose result depends on the order the threads take the symbols in fails here only on
    /// the runs where that order differs from the single thread's; each thread count is one more
    /// chance to catch it.
    #[test]
    fn test_optimizing_on_worker_threads_leaves_the_program_one_thread_leaves() {
        let source = r#"
module Main;

type Point = struct { x : I64, y : I64 };

sum_by : (a -> I64) -> Array a -> I64;
sum_by = |f, arr| arr.to_iter.fold(0, |e, acc| acc + f(e));

walk0 : I64 -> Point -> I64;
walk0 = |n, p| (
    let Point { x : x, y : y } = p;
    if n <= 0 { x + y * 0 };
    walk0(n - 1, Point { x : y, y : x + 0 })
);

walk1 : I64 -> Point -> I64;
walk1 = |n, p| (
    let Point { x : x, y : y } = p;
    if n <= 0 { x + y * 1 };
    walk1(n - 1, Point { x : y, y : x + 1 })
);

walk2 : I64 -> Point -> I64;
walk2 = |n, p| (
    let Point { x : x, y : y } = p;
    if n <= 0 { x + y * 2 };
    walk2(n - 1, Point { x : y, y : x + 2 })
);

walk3 : I64 -> Point -> I64;
walk3 = |n, p| (
    let Point { x : x, y : y } = p;
    if n <= 0 { x + y * 3 };
    walk3(n - 1, Point { x : y, y : x + 3 })
);

walk4 : I64 -> Point -> I64;
walk4 = |n, p| (
    let Point { x : x, y : y } = p;
    if n <= 0 { x + y * 4 };
    walk4(n - 1, Point { x : y, y : x + 4 })
);

walk5 : I64 -> Point -> I64;
walk5 = |n, p| (
    let Point { x : x, y : y } = p;
    if n <= 0 { x + y * 5 };
    walk5(n - 1, Point { x : y, y : x + 5 })
);

walk6 : I64 -> Point -> I64;
walk6 = |n, p| (
    let Point { x : x, y : y } = p;
    if n <= 0 { x + y * 6 };
    walk6(n - 1, Point { x : y, y : x + 6 })
);

walk7 : I64 -> Point -> I64;
walk7 = |n, p| (
    let Point { x : x, y : y } = p;
    if n <= 0 { x + y * 7 };
    walk7(n - 1, Point { x : y, y : x + 7 })
);

main : IO ();
main = (
    let points = Iterator::range(0, 100).map(|i| Point { x : i, y : i * i }).to_array;
    let total = points.sum_by(|p| p.@x + p.@y);
    let fact = fix(|f, n| if n == 0 { 1 } else { n * f(n - 1) });
    let walks = walk0(10, Point { x : 0, y : 2 }) + walk1(10, Point { x : 1, y : 2 }) + walk2(10, Point { x : 2, y : 2 }) + walk3(10, Point { x : 3, y : 2 }) + walk4(10, Point { x : 4, y : 2 }) + walk5(10, Point { x : 5, y : 2 }) + walk6(10, Point { x : 6, y : 2 }) + walk7(10, Point { x : 7, y : 2 });
    println(total.to_string + " " + fact(10).to_string + " " + walks.to_string)
);
"#;
        let (names, definitions) = optimized_symbols(source, 1);
        for num_threads in [2, 4, 8, 16] {
            let (names_on_threads, definitions_on_threads) = optimized_symbols(source, num_threads);
            assert_eq!(
                names, names_on_threads,
                "the program optimized on {} threads holds its symbols in another order",
                num_threads
            );
            assert_eq!(
                definitions, definitions_on_threads,
                "the program optimized on {} threads defines its symbols differently",
                num_threads
            );
        }
    }
}
