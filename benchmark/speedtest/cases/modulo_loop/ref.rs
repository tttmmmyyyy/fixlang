// The Rust counterpart of `main.fix`, on the same input, so the log can carry a
// reference the Fix line is read against. It checks the answer and prints nothing, as the
// Fix case does: a reference that computed something else would otherwise pass unnoticed.

fn loop_sum(n: i64) -> i64 {
    let mut acc = 0i64;
    for i in 0..n {
        acc = (acc + i) % 1_000_000_007;
    }
    acc
}

fn main() {
    assert_eq!(loop_sum(10_000_000), 994_650_007);
}
