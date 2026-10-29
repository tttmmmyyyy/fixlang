// The C counterpart of `main.fix`, on the same input, so the log can carry a
// reference the Fix line is read against. It checks the answer and prints nothing, as the
// Fix case does: a reference that computed something else would otherwise pass unnoticed.

#include <stdio.h>
#include <stdlib.h>

int main(void) {
    int n = 10;
    int perm[32], perm1[32], count[32];
    for (int i = 0; i < n; i++) perm1[i] = i;
    int r = n, maxflips = 0, checksum = 0, permcount = 0;
    while (1) {
        while (r != 1) { count[r - 1] = r; r--; }
        for (int i = 0; i < n; i++) perm[i] = perm1[i];
        int flips = 0, k;
        while ((k = perm[0]) != 0) {
            for (int i = 0, j = k; i < j; i++, j--) { int t = perm[i]; perm[i] = perm[j]; perm[j] = t; }
            flips++;
        }
        if (flips > maxflips) maxflips = flips;
        checksum += (permcount % 2 == 0) ? flips : -flips;
        // next permutation
        while (1) {
            if (r == n) {
                if (checksum != 73196 || maxflips != 38) {
                    fprintf(stderr, "fannkuch: %d %d\n", checksum, maxflips);
                    return 1;
                }
                return 0;
            }
            int perm0 = perm1[0];
            for (int i = 0; i < r; i++) perm1[i] = perm1[i + 1];
            perm1[r] = perm0;
            if (--count[r] > 0) break;
            r++;
        }
        permcount++;
    }
}
