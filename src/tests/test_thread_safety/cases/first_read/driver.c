#include <assert.h>
#include <pthread.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>

// The most threads this driver reads the global value from.
#define MAX_THREADS 64

// The number of elements of the global value.
#define SIZE 1000

// Defined by the Fix program through FFI_EXPORT: reads the global value and adds its elements to
// `start`.
int64_t fix_sum_squares(int64_t start);

// How many times the initializer of the global value has run.
static int runs;

// Called by the initializer of the global value. Counts the run, and sleeps so that the other
// threads reach the global value while its initializer is still running.
int start_initialization(void) {
    __atomic_add_fetch(&runs, 1, __ATOMIC_RELAXED);
    struct timespec pause = {0, 50 * 1000 * 1000};
    nanosleep(&pause, 0);
    return SIZE;
}

int initialization_runs(void) {
    return __atomic_load_n(&runs, __ATOMIC_RELAXED);
}

// POSIX leaves barriers optional and Darwin omits them, so the meeting point the threads need is
// built here out of a mutex and a condition variable, which every platform provides.
static pthread_mutex_t gate = PTHREAD_MUTEX_INITIALIZER;
static pthread_cond_t all_arrived = PTHREAD_COND_INITIALIZER;
static int participants;
static int arrived;

// Holds a thread until every thread has reached this point, so that the threads read the global
// value for the first time together.
static void rendezvous(void) {
    pthread_mutex_lock(&gate);
    arrived++;
    if (arrived == participants) pthread_cond_broadcast(&all_arrived);
    while (arrived < participants) pthread_cond_wait(&all_arrived, &gate);
    pthread_mutex_unlock(&gate);
}

static void *worker(void *unused) {
    (void)unused;
    rendezvous();
    int64_t sum = fix_sum_squares(0);
    // The sum of i * i for i in [0, SIZE).
    int64_t expected = (int64_t)(SIZE - 1) * SIZE * (2 * SIZE - 1) / 6;
    if (sum != expected) {
        fprintf(stderr, "a thread read the sum %lld, and %lld is expected\n", (long long)sum,
                (long long)expected);
        abort();
    }
    return 0;
}

// Reads the global value from `threads` threads at once, and waits for them.
void read_from_threads(int threads) {
    assert(threads >= 1 && threads <= MAX_THREADS);
    pthread_t ids[MAX_THREADS];
    participants = threads;
    for (int i = 0; i < threads; i++) pthread_create(&ids[i], 0, worker, 0);
    for (int i = 0; i < threads; i++) pthread_join(ids[i], 0);
}
