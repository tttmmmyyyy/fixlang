#[derive(Clone, Copy)]
struct Body { x: f64, y: f64, z: f64, vx: f64, vy: f64, vz: f64, mass: f64 }

const N: usize = 5;

fn init() -> [Body; N] {
    let pi = 3.141592653589793_f64;
    let solar_mass = 4.0 * pi * pi;
    let dpy = 365.24_f64;
    let mut b = [
        Body { x: 0.0, y: 0.0, z: 0.0, vx: 0.0, vy: 0.0, vz: 0.0, mass: solar_mass },
        Body { x: 4.84143144246472090e+00, y: -1.16032004402742839e+00, z: -1.03622044471123109e-01,
               vx: 1.66007664274403694e-03*dpy, vy: 7.69901118419740425e-03*dpy, vz: -6.90460016972063023e-05*dpy,
               mass: 9.54791938424326609e-04*solar_mass },
        Body { x: 8.34336671824457987e+00, y: 4.12479856412430479e+00, z: -4.03523417114321381e-01,
               vx: -2.76742510726862411e-03*dpy, vy: 4.99852801234917238e-03*dpy, vz: 2.30417297573763929e-05*dpy,
               mass: 2.85885980666130812e-04*solar_mass },
        Body { x: 1.28943695621391310e+01, y: -1.51111514016986312e+01, z: -2.23307578892655734e-01,
               vx: 2.96460137564761618e-03*dpy, vy: 2.37847173959480950e-03*dpy, vz: -2.96589568540237556e-05*dpy,
               mass: 4.36624404335156298e-05*solar_mass },
        Body { x: 1.53796971148509165e+01, y: -2.59193146099879641e+01, z: 1.79258772950371181e-01,
               vx: 2.68067772490389322e-03*dpy, vy: 1.62824170038242295e-03*dpy, vz: -9.51592254519715870e-05*dpy,
               mass: 1.62561576738991480e-04*solar_mass },
    ];
    let (mut px, mut py, mut pz) = (0.0, 0.0, 0.0);
    for i in 0..N { px += b[i].vx*b[i].mass; py += b[i].vy*b[i].mass; pz += b[i].vz*b[i].mass; }
    b[0].vx = -px/solar_mass; b[0].vy = -py/solar_mass; b[0].vz = -pz/solar_mass;
    b
}

fn energy(b: &[Body; N]) -> f64 {
    let mut e = 0.0;
    for i in 0..N {
        e += 0.5*b[i].mass*(b[i].vx*b[i].vx + b[i].vy*b[i].vy + b[i].vz*b[i].vz);
        for j in i+1..N {
            let (dx, dy, dz) = (b[i].x-b[j].x, b[i].y-b[j].y, b[i].z-b[j].z);
            e -= b[i].mass*b[j].mass/(dx*dx+dy*dy+dz*dz).sqrt();
        }
    }
    e
}

fn advance(b: &mut [Body; N], dt: f64) {
    for i in 0..N { for j in i+1..N {
        let (dx, dy, dz) = (b[i].x-b[j].x, b[i].y-b[j].y, b[i].z-b[j].z);
        let d2 = dx*dx+dy*dy+dz*dz; let d = d2.sqrt(); let mag = dt/(d2*d);
        let (mi, mj) = (b[i].mass, b[j].mass);
        b[i].vx -= dx*mj*mag; b[i].vy -= dy*mj*mag; b[i].vz -= dz*mj*mag;
        b[j].vx += dx*mi*mag; b[j].vy += dy*mi*mag; b[j].vz += dz*mi*mag;
    } }
    for i in 0..N { b[i].x += dt*b[i].vx; b[i].y += dt*b[i].vy; b[i].z += dt*b[i].vz; }
}

fn main() {
    let steps: i64 = std::env::args().last().unwrap().parse().unwrap();
    let mut b = init();
    for _ in 0..steps { advance(&mut b, 0.01); }
    println!("{:.9}", energy(&b));
}
