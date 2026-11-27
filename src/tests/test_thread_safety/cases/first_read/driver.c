#include <assert.h>
#include <pthread.h>
#include <stdint.h>
#include <stdio.h>
#include <sched.h>
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

// Set by the first thread of the waiting group to finish reading the global value. It is written
// and read with relaxed ordering, so that a thread of the late group finding it set has no
// happens-before edge to the initializing thread other than the one the global's accessor makes.
static int read_done;

static void check(int64_t sum) {
    // The sum of i * i for i in [0, SIZE).
    int64_t expected = (int64_t)(SIZE - 1) * SIZE * (2 * SIZE - 1) / 6;
    if (sum != expected) {
        fprintf(stderr, "a thread read the sum %lld, and %lld is expected\n", (long long)sum,
                (long long)expected);
        abort();
    }
}

// A thread of the waiting group: arrives with the others while the initializer is running.
static void *waiting_reader(void *unused) {
    (void)unused;
    rendezvous();
    check(fix_sum_squares(0));
    __atomic_store_n(&read_done, 1, __ATOMIC_RELAXED);
    return 0;
}

// A thread of the late group: reads the global value once a thread of the waiting group has read
// it, and so finds the initialization flag set.
static void *late_reader(void *unused) {
    (void)unused;
    while (!__atomic_load_n(&read_done, __ATOMIC_RELAXED)) sched_yield();
    check(fix_sum_squares(0));
    return 0;
}

// Reads the global value from two groups of `threads` threads, and waits for them. The waiting
// group reaches the global value while its initializer is running, and the late group after it
// has finished.
void read_from_threads(int threads) {
    assert(threads >= 1 && 2 * threads <= MAX_THREADS);
    pthread_t ids[MAX_THREADS];
    participants = threads;
    for (int i = 0; i < threads; i++) pthread_create(&ids[i], 0, late_reader, 0);
    for (int i = threads; i < 2 * threads; i++) pthread_create(&ids[i], 0, waiting_reader, 0);
    for (int i = 0; i < 2 * threads; i++) pthread_join(ids[i], 0);
}
