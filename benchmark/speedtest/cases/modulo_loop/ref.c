// The C counterpart of `main.fix`, on the same input, so the log can carry a
// reference the Fix line is read against. It checks the answer and prints nothing, as the
// Fix case does: a reference that computed something else would otherwise pass unnoticed.

#include <stdint.h>
#include <stdio.h>

static int64_t loop_sum(int64_t n) {
    int64_t acc = 0;
    for (int64_t i = 0; i < n; i++) acc = (acc + i) % 1000000007;
    return acc;
}

int main(void) {
    int64_t acc = loop_sum(10000000);
    if (acc != 994650007) {
        fprintf(stderr, "modulo_loop: %lld\n", (long long)acc);
        return 1;
    }
    return 0;
}
