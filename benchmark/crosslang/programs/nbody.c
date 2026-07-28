#include <stdio.h>
#include <stdlib.h>
#include <math.h>

typedef struct { double x, y, z, vx, vy, vz, mass; } Body;
#define N 5

static Body b[N];

static void init(void) {
    const double pi = 3.141592653589793;
    const double solar_mass = 4.0 * pi * pi;
    const double dpy = 365.24;
    Body init_b[N] = {
        {0,0,0, 0,0,0, solar_mass},
        {4.84143144246472090e+00, -1.16032004402742839e+00, -1.03622044471123109e-01,
         1.66007664274403694e-03*dpy, 7.69901118419740425e-03*dpy, -6.90460016972063023e-05*dpy,
         9.54791938424326609e-04*solar_mass},
        {8.34336671824457987e+00, 4.12479856412430479e+00, -4.03523417114321381e-01,
         -2.76742510726862411e-03*dpy, 4.99852801234917238e-03*dpy, 2.30417297573763929e-05*dpy,
         2.85885980666130812e-04*solar_mass},
        {1.28943695621391310e+01, -1.51111514016986312e+01, -2.23307578892655734e-01,
         2.96460137564761618e-03*dpy, 2.37847173959480950e-03*dpy, -2.96589568540237556e-05*dpy,
         4.36624404335156298e-05*solar_mass},
        {1.53796971148509165e+01, -2.59193146099879641e+01, 1.79258772950371181e-01,
         2.68067772490389322e-03*dpy, 1.62824170038242295e-03*dpy, -9.51592254519715870e-05*dpy,
         1.62561576738991480e-04*solar_mass},
    };
    for (int i = 0; i < N; i++) b[i] = init_b[i];
    double px = 0, py = 0, pz = 0;
    for (int i = 0; i < N; i++) { px += b[i].vx*b[i].mass; py += b[i].vy*b[i].mass; pz += b[i].vz*b[i].mass; }
    b[0].vx = -px/solar_mass; b[0].vy = -py/solar_mass; b[0].vz = -pz/solar_mass;
}

static double energy(void) {
    double e = 0;
    for (int i = 0; i < N; i++) {
        e += 0.5*b[i].mass*(b[i].vx*b[i].vx + b[i].vy*b[i].vy + b[i].vz*b[i].vz);
        for (int j = i+1; j < N; j++) {
            double dx = b[i].x-b[j].x, dy = b[i].y-b[j].y, dz = b[i].z-b[j].z;
            e -= b[i].mass*b[j].mass/sqrt(dx*dx+dy*dy+dz*dz);
        }
    }
    return e;
}

static void advance(double dt) {
    for (int i = 0; i < N; i++) for (int j = i+1; j < N; j++) {
        double dx = b[i].x-b[j].x, dy = b[i].y-b[j].y, dz = b[i].z-b[j].z;
        double d2 = dx*dx+dy*dy+dz*dz, d = sqrt(d2), mag = dt/(d2*d);
        b[i].vx -= dx*b[j].mass*mag; b[i].vy -= dy*b[j].mass*mag; b[i].vz -= dz*b[j].mass*mag;
        b[j].vx += dx*b[i].mass*mag; b[j].vy += dy*b[i].mass*mag; b[j].vz += dz*b[i].mass*mag;
    }
    for (int i = 0; i < N; i++) { b[i].x += dt*b[i].vx; b[i].y += dt*b[i].vy; b[i].z += dt*b[i].vz; }
}

int main(int argc, char **argv) {
    long steps = atoll(argv[argc - 1]);
    init();
    for (long s = 0; s < steps; s++) advance(0.01);
    printf("%.9f\n", energy());
    return 0;
}
