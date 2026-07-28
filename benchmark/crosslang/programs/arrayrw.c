#include <stdio.h>
#include <stdlib.h>

int main(int argc, char **argv) {
    long iters = atoll(argv[argc - 1]);
    static long arr[1000];
    for (int i = 0; i < 1000; i++) arr[i] = 0;
    for (long k = 0; k < iters; k++)
        for (int i = 0; i < 1000; i++) arr[i] = arr[i] + 1;
    printf("%ld\n", arr[0]);
    return 0;
}
