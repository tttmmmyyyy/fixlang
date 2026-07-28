use std::hint::black_box;
use std::time::Instant;

fn loopsum(n: i64) -> i64 {
    let mut acc: i64 = 0;
    let mut i: i64 = 0;
    while i < n {
        acc = (acc + i) % 1000000007;
        i += 1;
    }
    acc
}

fn main() {
    let n: i64 = std::env::args().last().unwrap().parse().unwrap();
    let t0 = Instant::now();
    let ans = black_box(loopsum(black_box(n)));
    let ns = t0.elapsed().as_nanos();
    println!("rust,loop,{},{}", ns, ans);
}
