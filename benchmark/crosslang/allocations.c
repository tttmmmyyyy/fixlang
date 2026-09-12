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

// Every entry the C library offers, since a language reaches the allocator through whichever
// one its runtime picked: Rust's `vec![0; n]` arrives as `calloc` by way of
// `__rust_alloc_zeroed`, and a counterpart counted through `malloc` alone reads as allocating
// half as often as the case beside it.
//
// `dlsym` resolves the real entry on the first call. Where a C library reaches for the allocator
// while it resolves, that first call recurses; glibc does not, and a library that does would
// need the real entries resolved from a constructor into a static buffer instead.
static long n_malloc, n_calloc, n_realloc, n_aligned, n_posix_memalign;
static void *(*real_malloc)(size_t);
static void *(*real_calloc)(size_t, size_t);
static void *(*real_realloc)(void *, size_t);
static void *(*real_aligned_alloc)(size_t, size_t);
static int (*real_posix_memalign)(void **, size_t, size_t);

void *malloc(size_t size) {
    if (!real_malloc) real_malloc = dlsym(RTLD_NEXT, "malloc");
    n_malloc++;
    return real_malloc(size);
}

void *calloc(size_t count, size_t size) {
    if (!real_calloc) real_calloc = dlsym(RTLD_NEXT, "calloc");
    n_calloc++;
    return real_calloc(count, size);
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

int posix_memalign(void **out, size_t alignment, size_t size) {
    if (!real_posix_memalign) real_posix_memalign = dlsym(RTLD_NEXT, "posix_memalign");
    n_posix_memalign++;
    return real_posix_memalign(out, alignment, size);
}

__attribute__((destructor)) static void report(void) {
    fprintf(stderr, "malloc=%ld calloc=%ld realloc=%ld aligned_alloc=%ld posix_memalign=%ld\n",
            n_malloc, n_calloc, n_realloc, n_aligned, n_posix_memalign);
}
