#include <assert.h>
#include <pthread.h>

// The most threads this driver hands a value to.
#define MAX_THREADS 64

// Takes a pointer and does nothing with it. The Fix code calls this between taking a reference and
// dropping it, so that the pair escapes the optimizer's view and survives into the running program.


// Defined by the Fix program through FFI_EXPORT: takes one reference to the shared value, works it,
// and drops it.
void fix_hammer_one(void *value);

static pthread_barrier_t start;

static void *worker(void *value) {
    // Wait for every thread to arrive, so that the reference counting they do overlaps. Without
    // this each thread finishes before the next is created and nothing contends.
    pthread_barrier_wait(&start);
    fix_hammer_one(value);
    return 0;
}

// Hands `value` to `threads` threads, each holding one reference of its own, and waits for them.
void hammer_from_threads(void *value, int threads) {
    assert(threads >= 1 && threads <= MAX_THREADS);
    pthread_t ids[MAX_THREADS];
    pthread_barrier_init(&start, 0, threads);
    for (int i = 0; i < threads; i++) pthread_create(&ids[i], 0, worker, value);
    for (int i = 0; i < threads; i++) pthread_join(ids[i], 0);
    pthread_barrier_destroy(&start);
}
