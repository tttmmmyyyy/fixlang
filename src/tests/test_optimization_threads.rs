// The optimization passes rewrite the symbols of a program on the compiler's worker threads. The
// program they leave has to be the one a single thread leaves: the same symbols, defined the same
// way, held in the same order, since the stages after them generate code in that order.

#[cfg(test)]
mod tests {
    use crate::elaboration::elaborate_at_max;
    use crate::optimization::optimization;

    /// The names of the symbols `source` is optimized into, in the order the program holds them, and
    /// the text of their definitions, with the passes running on `num_threads` threads.
    fn optimized_symbols(source: &str, num_threads: usize) -> (Vec<String>, String) {
        let (mut program, config) = elaborate_at_max(source, "optimization_threads", num_threads);
        optimization::run(&mut program, &config);
        let names = program
            .symbols
            .keys()
            .map(|name| name.to_string())
            .collect();
        (names, program.stringify_symbols().to_string())
    }

    /// Optimizing on several threads leaves the same program as optimizing on one thread, for a
    /// program that every pass running on the worker threads rewrites: globals to inline, functions
    /// to uncurry, functions taking lambdas to copy for their lambda sets, and constructions to read.
    ///
    /// A pass whose result depends on the order the threads take the symbols in fails here only on
    /// the runs where that order differs from the single thread's; each thread count is one more
    /// chance to catch it.
    #[test]
    fn test_optimizing_on_worker_threads_leaves_the_program_one_thread_leaves() {
        let source = r#"
module Main;

sum_by : (a -> I64) -> Array a -> I64;
sum_by = |f, arr| arr.to_iter.fold(0, |e, acc| acc + f(e));

walk0 : I64 -> (I64 -> I64) -> I64 -> I64;
walk0 = |n, f, x| if n <= 0 { f(x) } else { walk0(n - 1, f, x + 0) };

walk1 : I64 -> (I64 -> I64) -> I64 -> I64;
walk1 = |n, f, x| if n <= 0 { f(x) } else { walk1(n - 1, f, x + 1) };

walk2 : I64 -> (I64 -> I64) -> I64 -> I64;
walk2 = |n, f, x| if n <= 0 { f(x) } else { walk2(n - 1, f, x + 2) };

walk3 : I64 -> (I64 -> I64) -> I64 -> I64;
walk3 = |n, f, x| if n <= 0 { f(x) } else { walk3(n - 1, f, x + 3) };

walk4 : I64 -> (I64 -> I64) -> I64 -> I64;
walk4 = |n, f, x| if n <= 0 { f(x) } else { walk4(n - 1, f, x + 4) };

walk5 : I64 -> (I64 -> I64) -> I64 -> I64;
walk5 = |n, f, x| if n <= 0 { f(x) } else { walk5(n - 1, f, x + 5) };

walk6 : I64 -> (I64 -> I64) -> I64 -> I64;
walk6 = |n, f, x| if n <= 0 { f(x) } else { walk6(n - 1, f, x + 6) };

walk7 : I64 -> (I64 -> I64) -> I64 -> I64;
walk7 = |n, f, x| if n <= 0 { f(x) } else { walk7(n - 1, f, x + 7) };

main : IO ();
main = (
    let points = Iterator::range(0, 100).map(|i| (i, i * i)).to_array;
    let total = points.sum_by(|(x, y)| x + y);
    let fact = fix(|f, n| if n == 0 { 1 } else { n * f(n - 1) });
    let walks = walk0(10, |v| v * 1, 0) + walk1(10, |v| v * 2, 1) + walk2(10, |v| v * 3, 2) + walk3(10, |v| v * 4, 3) + walk4(10, |v| v * 5, 4) + walk5(10, |v| v * 6, 5) + walk6(10, |v| v * 7, 6) + walk7(10, |v| v * 8, 7);
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
