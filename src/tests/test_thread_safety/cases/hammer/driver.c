#include <assert.h>
#include <pthread.h>

// The most threads this driver hands a value to.
#define MAX_THREADS 64

// Defined by the Fix program through FFI_EXPORT: takes one reference to the shared value, works it,
// and drops it.
void fix_hammer_one(void *value);

// POSIX leaves barriers optional and Darwin omits them, so the meeting point the threads need is
// built here out of a mutex and a condition variable, which every platform provides.
static pthread_mutex_t gate = PTHREAD_MUTEX_INITIALIZER;
static pthread_cond_t all_arrived = PTHREAD_COND_INITIALIZER;
static int expected;
static int arrived;

// Holds a thread until every thread has reached this point, so that the reference counting they do
// overlaps. Without this each thread finishes before the next is created and nothing contends.
//
// Every thread passes here once, so a thread waits for the arrivals to reach `expected` and nothing
// resets the count.
static void rendezvous(void) {
    pthread_mutex_lock(&gate);
    arrived++;
    if (arrived == expected) pthread_cond_broadcast(&all_arrived);
    while (arrived < expected) pthread_cond_wait(&all_arrived, &gate);
    pthread_mutex_unlock(&gate);
}

static void *worker(void *value) {
    rendezvous();
    fix_hammer_one(value);
    return 0;
}

// Hands `value` to `threads` threads, each holding one reference of its own, and waits for them.
void hammer_from_threads(void *value, int threads) {
    assert(threads >= 1 && threads <= MAX_THREADS);
    pthread_t ids[MAX_THREADS];
    expected = threads;
    for (int i = 0; i < threads; i++) pthread_create(&ids[i], 0, worker, value);
    for (int i = 0; i < threads; i++) pthread_join(ids[i], 0);
}
