#include <stdio.h>
#include <stdlib.h>

static long rand_state = 1;

static long next_rand(void) {
    rand_state = (16807 * rand_state) % 2147483647;
    return rand_state;
}

static long distance(const char *a, long m, const char *b, long n) {
    if (m > n) {
        const char *t = a; a = b; b = t;
        long tl = m; m = n; n = tl;
    }
    long *prev = malloc((m + 1) * sizeof(long));
    long *cur = malloc((m + 1) * sizeof(long));
    for (long i = 0; i <= m; i++) prev[i] = i;
    for (long j = 1; j <= n; j++) {
        cur[0] = j;
        for (long i = 1; i <= m; i++) {
            long cost = a[i - 1] == b[j - 1] ? 0 : 1;
            long d = prev[i] + 1;
            if (cur[i - 1] + 1 < d) d = cur[i - 1] + 1;
            if (prev[i - 1] + cost < d) d = prev[i - 1] + cost;
            cur[i] = d;
        }
        long *t = prev; prev = cur; cur = t;
    }
    long d = prev[m];
    free(prev);
    free(cur);
    return d;
}

int main(int argc, char **argv) {
    long n = atoll(argv[argc - 1]);
    char **words = malloc(n * sizeof(char *));
    long *lens = malloc(n * sizeof(long));
    for (long k = 0; k < n; k++) {
        long len = 3 + next_rand() % 8;
        char *w = malloc(len);
        for (long i = 0; i < len; i++) w[i] = 'a' + next_rand() % 26;
        words[k] = w;
        lens[k] = len;
    }
    long sum = 0;
    for (long i = 0; i < n; i++)
        for (long j = i + 1; j < n; j++)
            sum += distance(words[i], lens[i], words[j], lens[j]);
    printf("%ld\n", sum);
    return 0;
}
