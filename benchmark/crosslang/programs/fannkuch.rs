fn main() {
    let n: usize = std::env::args().last().unwrap().parse().unwrap();
    let mut perm = [0i32; 32];
    let mut perm1 = [0i32; 32];
    let mut count = [0i32; 32];
    for i in 0..n {
        perm1[i] = i as i32;
    }
    let mut r = n;
    let mut maxflips = 0i32;
    let mut checksum = 0i32;
    let mut permcount = 0i32;
    loop {
        while r != 1 {
            count[r - 1] = r as i32;
            r -= 1;
        }
        perm[..n].copy_from_slice(&perm1[..n]);
        let mut flips = 0i32;
        loop {
            let k = perm[0] as usize;
            if k == 0 {
                break;
            }
            let (mut i, mut j) = (0usize, k);
            while i < j {
                perm.swap(i, j);
                i += 1;
                j -= 1;
            }
            flips += 1;
        }
        if flips > maxflips {
            maxflips = flips;
        }
        checksum += if permcount % 2 == 0 { flips } else { -flips };
        loop {
            if r == n {
                println!("{} {}", checksum, maxflips);
                return;
            }
            let perm0 = perm1[0];
            for i in 0..r {
                perm1[i] = perm1[i + 1];
            }
            perm1[r] = perm0;
            count[r] -= 1;
            if count[r] > 0 {
                break;
            }
            r += 1;
        }
        permcount += 1;
    }
}
