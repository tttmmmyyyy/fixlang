// The Rust counterpart of `main.fix`, on the same input, so the log can carry a
// reference the Fix line is read against. It checks the answer and prints nothing, as the
// Fix case does: a reference that computed something else would otherwise pass unnoticed.

use std::hint::black_box;

fn fib(n: i64) -> i64 {
    if n <= 1 { n } else { fib(n - 1) + fib(n - 2) }
}

fn main() {
    // `black_box` keeps the compiler from folding the whole call tree to a constant, which
    // is what the Fix case gets from taking `n` off the argument count.
    assert_eq!(fib(black_box(34)), 5_702_887);
}
