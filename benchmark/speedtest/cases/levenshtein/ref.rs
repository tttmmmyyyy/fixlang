// The Rust counterpart of `main.fix`, on the same input, so the log can carry a
// reference the Fix line is read against. It checks the answer and prints nothing, as the
// Fix case does: a reference that computed something else would otherwise pass unnoticed.

fn next_rand(state: &mut i64) -> i64 {
    *state = (16807 * *state) % 2147483647;
    *state
}

fn distance(a: &[u8], b: &[u8]) -> i64 {
    let (a, b) = if a.len() <= b.len() { (a, b) } else { (b, a) };
    let m = a.len();
    let n = b.len();
    let mut prev: Vec<i64> = (0..=m as i64).collect();
    let mut cur: Vec<i64> = vec![0; m + 1];
    for j in 1..=n {
        cur[0] = j as i64;
        for i in 1..=m {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            let mut d = prev[i] + 1;
            if cur[i - 1] + 1 < d {
                d = cur[i - 1] + 1;
            }
            if prev[i - 1] + cost < d {
                d = prev[i - 1] + cost;
            }
            cur[i] = d;
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[m]
}

fn main() {
    let n: usize = 1000;
    let mut state = 1i64;
    let mut words: Vec<Vec<u8>> = Vec::with_capacity(n);
    for _ in 0..n {
        let len = 3 + next_rand(&mut state) % 8;
        let mut w: Vec<u8> = Vec::with_capacity(len as usize);
        for _ in 0..len {
            w.push(b'a' + (next_rand(&mut state) % 26) as u8);
        }
        words.push(w);
    }
    let mut sum = 0i64;
    for i in 0..n {
        for j in i + 1..n {
            sum += distance(&words[i], &words[j]);
        }
    }
    assert_eq!(sum, 3_648_154);
}
