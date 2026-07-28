// Timing + optimization-barrier helpers for the Fix benchmarks.
//
// Fix code is pure, so the optimizer is free to move a pure computation
// (e.g. `fib(n)`) out of the timed region. To pin the work between the two
// clock reads we route the input and the result through opaque FFI calls that
// touch a `volatile` global, which the optimizer must treat as a real
// side effect it cannot reorder or fold away.

#include <time.h>
#include <stdint.h>

static volatile int64_t g_sink = 0;

// Monotonic clock in nanoseconds. Used for the start timestamp.
int64_t mono_nanos(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (int64_t)ts.tv_sec * 1000000000LL + ts.tv_nsec;
}

// Identity on `x` (g_sink is always 0), but opaque to the optimizer.
// Calling this after the start timestamp forces the work that depends on its
// result to be scheduled after the start timestamp.
int64_t opaque_i64(int64_t x) {
    return x + g_sink;
}

// Consumes `x` (stores it to the volatile global) and returns the monotonic
// clock. Used for the end timestamp: passing the work's result as `x` forces
// the work to complete before this clock read.
int64_t sink_then_mono(int64_t x) {
    g_sink = x;
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (int64_t)ts.tv_sec * 1000000000LL + ts.tv_nsec;
}
