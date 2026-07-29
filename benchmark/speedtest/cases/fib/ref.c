// The C counterpart of `main.fix`, on the same input, so the log can carry a
// reference the Fix line is read against. It checks the answer and prints nothing, as the
// Fix case does: a reference that computed something else would otherwise pass unnoticed.

#include <stdint.h>
#include <stdio.h>

static int64_t fib(int64_t n) {
    return n <= 1 ? n : fib(n - 1) + fib(n - 2);
}

int main(void) {
    // `volatile` keeps the compiler from folding the whole call tree to a constant, which
    // is what the Fix case gets from taking `n` off the argument count.
    volatile int64_t n = 34;
    int64_t answer = fib(n);
    if (answer != 5702887) {
        fprintf(stderr, "fib: %lld\n", (long long)answer);
        return 1;
    }
    return 0;
}
