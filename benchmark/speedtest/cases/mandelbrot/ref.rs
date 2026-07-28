// The Rust counterpart of `main.fix`, on the same input, so the log can carry a
// reference the Fix line is read against. It checks the answer and prints nothing, as the
// Fix case does: a reference that computed something else would otherwise pass unnoticed.

fn in_set(cr: f64, ci: f64) -> i64 {
    let mut zr = 0.0_f64;
    let mut zi = 0.0_f64;
    let mut iter = 0;
    while iter < 50 {
        if zr * zr + zi * zi > 4.0 {
            return 0;
        }
        let zr2 = zr * zr - zi * zi + cr;
        let zi2 = 2.0 * zr * zi + ci;
        zr = zr2;
        zi = zi2;
        iter += 1;
    }
    1
}

fn main() {
    let n: i64 = 1500;
    let step = 0.0025_f64;
    let mut count: i64 = 0;
    let mut ci = -1.0_f64;
    let mut py = 0;
    while py < n {
        let mut cr = -2.0_f64;
        let mut px = 0;
        while px < n {
            count += in_set(cr, ci);
            cr += step;
            px += 1;
        }
        ci += step;
        py += 1;
    }
    assert_eq!(count, 254662);
}
