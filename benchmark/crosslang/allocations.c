// Count the allocations a program makes, by interposing on the allocator.
//
// "How many times does this allocate" is a question a reference-counted language raises
// constantly, and neither the instruction count nor the wall clock answers it directly.
// Two defects in these benchmarks were found this way: a counterpart allocating half as
// often as the case it was compared against, and a case allocating once per iteration
// where its counterparts allocate not at all.
//
//   gcc -shared -fPIC -O2 allocations.c -o allocations.so -ldl
//   LD_PRELOAD=$PWD/allocations.so ./bin/fannkuch_fix
//
// The counts are written to stderr when the program exits.

#define _GNU_SOURCE
#include <dlfcn.h>
#include <stdio.h>
#include <stdlib.h>

static long n_malloc, n_realloc, n_aligned;
static void *(*real_malloc)(size_t);
static void *(*real_realloc)(void *, size_t);
static void *(*real_aligned_alloc)(size_t, size_t);

void *malloc(size_t size) {
    if (!real_malloc) real_malloc = dlsym(RTLD_NEXT, "malloc");
    n_malloc++;
    return real_malloc(size);
}

void *realloc(void *ptr, size_t size) {
    if (!real_realloc) real_realloc = dlsym(RTLD_NEXT, "realloc");
    n_realloc++;
    return real_realloc(ptr, size);
}

void *aligned_alloc(size_t alignment, size_t size) {
    if (!real_aligned_alloc) real_aligned_alloc = dlsym(RTLD_NEXT, "aligned_alloc");
    n_aligned++;
    return real_aligned_alloc(alignment, size);
}

__attribute__((destructor)) static void report(void) {
    fprintf(stderr, "malloc=%ld realloc=%ld aligned_alloc=%ld\n",
            n_malloc, n_realloc, n_aligned);
}
