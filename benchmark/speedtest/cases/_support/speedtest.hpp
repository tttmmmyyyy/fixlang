// What the C++ counterparts of the cases that count part of themselves share, as
// `speedtest.fix` is for the Fix cases: the input they generate, the way they tell
// `perf_counters.py` which part of the run to count, and the check of the answer.
#include <cstdarg>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <vector>

namespace speedtest {

// `len` pseudo-random values in [0, m), for `m` at most 2^32, generated as `rands` in
// `speedtest.fix` generates them.
inline std::vector<long long> rands(long long len, long long m, unsigned long long seed) {
    std::vector<long long> v;
    v.reserve(len);
    unsigned long long r = seed;
    for (long long i = 0; i < len; i++) {
        r = r * 6364136223846793005ULL + 1442695040888963407ULL;
        v.push_back((long long)(((r >> 32) * (unsigned long long)m) >> 32));
    }
    return v;
}

// The FIFOs `perf_counters.py` names in `SPEEDTEST_PERF_CTL` and `SPEEDTEST_PERF_ACK`, as
// `PerfControl` in `speedtest.fix` holds them. Null outside the harness.
inline FILE* perf_ctl = nullptr;
inline FILE* perf_ack = nullptr;

// Send `command` to perf, and wait for its answer: `ack\n` and the NUL that ends it, read as the
// bytes they are, as `speedtest.fix` reads them.
inline void perf_send(const char* command) {
    if (!perf_ctl) return;
    fputs(command, perf_ctl);
    fflush(perf_ctl);
    char answer[sizeof "ack\n"];
    if (fread(answer, 1, sizeof answer, perf_ack) != sizeof answer) exit(1);
}

// Open the FIFOs and stop the counters, as `PerfControl::start` does.
inline void perf_start() {
    const char* ctl = getenv("SPEEDTEST_PERF_CTL");
    const char* ack = getenv("SPEEDTEST_PERF_ACK");
    if (!ctl || !ack) return;
    perf_ctl = fopen(ctl, "w");
    perf_ack = fopen(ack, "r");
    if (!perf_ctl || !perf_ack) exit(1);
    perf_send("disable\n");
}

// Count what runs between these two, as `PerfControl::measure` counts the evaluation of its work.
// The Fix program frees what its work built and does not return before `measure` ends, so a
// counterpart builds those in a block that closes before `perf_disable`, and only the result
// outlives it.
inline void perf_enable() { perf_send("enable\n"); }
inline void perf_disable() { perf_send("disable\n"); }

// Format the answer as `main.fix` does and compare it with `expected`. A mismatch is written to
// stderr, and the value returned is what `main` exits with.
inline int check(const char* expected, const char* fmt, ...) {
    char answer[256];
    va_list ap;
    va_start(ap, fmt);
    vsnprintf(answer, sizeof answer, fmt, ap);
    va_end(ap);
    if (strcmp(answer, expected) != 0) {
        fprintf(stderr, "expected %s, got %s", expected, answer);
        return 1;
    }
    return 0;
}

}  // namespace speedtest
