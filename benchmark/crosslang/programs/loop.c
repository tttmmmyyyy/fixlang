#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#include <stdint.h>

static int64_t loopsum(int64_t n) {
    int64_t acc = 0;
    for (int64_t i = 0; i < n; i++) {
        acc = (acc + i) % 1000000007;
    }
    return acc;
}

static int64_t mono_nanos(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (int64_t)ts.tv_sec * 1000000000LL + ts.tv_nsec;
}

int main(int argc, char **argv) {
    volatile int64_t n = atoll(argv[argc - 1]);
    int64_t t0 = mono_nanos();
    volatile int64_t ans = loopsum(n);
    int64_t t1 = mono_nanos();
    printf("c,loop,%lld,%lld\n", (long long)(t1 - t0), (long long)ans);
    return 0;
}
