// The C counterpart of `main.fix`, on the same input, so the log can carry a
// reference the Fix line is read against. It checks the answer and prints nothing, as the
// Fix case does: a reference that computed something else would otherwise pass unnoticed.

#include <stdio.h>
#include <stdlib.h>

int main(void) {
    long iters = 200000;
    static long arr[1000];
    for (int i = 0; i < 1000; i++) arr[i] = 0;
    for (long k = 0; k < iters; k++)
        for (int i = 0; i < 1000; i++) arr[i] = arr[i] + 1;
    if (arr[0] != 200000) { fprintf(stderr, "arrayrw: %ld\n", arr[0]); return 1; }
    return 0;
}
