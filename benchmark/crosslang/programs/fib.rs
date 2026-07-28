use std::hint::black_box;
use std::time::Instant;

fn fib(n: i64) -> i64 {
    if n <= 1 { n } else { fib(n - 1) + fib(n - 2) }
}

fn main() {
    let n: i64 = std::env::args().last().unwrap().parse().unwrap();
    let t0 = Instant::now();
    let ans = black_box(fib(black_box(n)));
    let ns = t0.elapsed().as_nanos();
    println!("rust,fib,{},{}", ns, ans);
}
