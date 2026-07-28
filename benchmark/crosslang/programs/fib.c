#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#include <stdint.h>

static int64_t fib(int64_t n) {
    return n <= 1 ? n : fib(n - 1) + fib(n - 2);
}

static int64_t mono_nanos(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (int64_t)ts.tv_sec * 1000000000LL + ts.tv_nsec;
}

int main(int argc, char **argv) {
    volatile int64_t n = atoll(argv[argc - 1]);
    int64_t t0 = mono_nanos();
    volatile int64_t ans = fib(n);
    int64_t t1 = mono_nanos();
    printf("c,fib,%lld,%lld\n", (long long)(t1 - t0), (long long)ans);
    return 0;
}
