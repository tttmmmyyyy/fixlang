#include <stdio.h>
#include <stdlib.h>

static int in_set(double cr, double ci) {
    double zr = 0.0, zi = 0.0;
    int iter = 0;
    while (iter < 50) {
        if (zr * zr + zi * zi > 4.0) return 0;
        double zr2 = zr * zr - zi * zi + cr;
        double zi2 = 2.0 * zr * zi + ci;
        zr = zr2; zi = zi2; iter++;
    }
    return 1;
}

int main(int argc, char **argv) {
    long n = atoll(argv[argc - 1]);
    double step = 0.0025;
    long count = 0;
    double ci = -1.0;
    for (long py = 0; py < n; py++) {
        double cr = -2.0;
        for (long px = 0; px < n; px++) { count += in_set(cr, ci); cr += step; }
        ci += step;
    }
    printf("%ld\n", count);
    return 0;
}
