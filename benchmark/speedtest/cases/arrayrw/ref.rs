// The Rust counterpart of `main.fix`, on the same input, so the log can carry a
// reference the Fix line is read against. It checks the answer and prints nothing, as the
// Fix case does: a reference that computed something else would otherwise pass unnoticed.

fn main() {
    let iters: i64 = 200_000;
    let mut arr = vec![0i64; 1000];
    for _ in 0..iters {
        for i in 0..1000 {
            arr[i] = arr[i] + 1;
        }
    }
    assert_eq!(arr[0], 200_000);
}
