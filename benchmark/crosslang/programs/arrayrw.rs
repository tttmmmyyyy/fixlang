fn main() {
    let iters: i64 = std::env::args().last().unwrap().parse().unwrap();
    let mut arr = vec![0i64; 1000];
    for _ in 0..iters {
        for i in 0..1000 {
            arr[i] = arr[i] + 1;
        }
    }
    println!("{}", arr[0]);
}
