// Loads the library built from `main.fix` and calls the function it exports on the given argument,
// printing the answer.
//
// Taking the library by path and reaching its function through `dlsym` is what a program loading a
// Fix library does, and it needs the exported name to be in the library's dynamic symbol table.

#include <dlfcn.h>
#include <stdio.h>
#include <stdlib.h>

// The type of `Main::sum_point`, which `FFI_EXPORT` offers as `fix_sum_point`.
typedef long long (*sum_point_fn)(long long);

int main(int argc, char **argv) {
    if (argc < 3) {
        fprintf(stderr, "usage: driver <library> <argument>\n");
        return 2;
    }
    void *library = dlopen(argv[1], RTLD_NOW);
    if (library == NULL) {
        fprintf(stderr, "dlopen failed: %s\n", dlerror());
        return 1;
    }
    sum_point_fn sum_point = (sum_point_fn)dlsym(library, "fix_sum_point");
    if (sum_point == NULL) {
        fprintf(stderr, "dlsym failed: %s\n", dlerror());
        return 1;
    }
    printf("%lld\n", sum_point(atoll(argv[2])));
    return 0;
}
