// Reads the value the library built from `main.fix` exports, and prints it.
//
// The value is not a function, so `FFI_EXPORT` publishes it as a C function of no arguments that
// answers what the value's initializer computes.

#include <dlfcn.h>
#include <stdio.h>

typedef int (*answer_fn)(void);

int main(int argc, char **argv) {
    if (argc < 2) {
        fprintf(stderr, "usage: driver <library>\n");
        return 2;
    }
    void *library = dlopen(argv[1], RTLD_NOW);
    if (library == NULL) {
        fprintf(stderr, "dlopen failed: %s\n", dlerror());
        return 1;
    }
    answer_fn answer = (answer_fn)dlsym(library, "fix_answer");
    if (answer == NULL) {
        fprintf(stderr, "dlsym failed: %s\n", dlerror());
        return 1;
    }
    printf("%d\n", answer());
    return 0;
}
