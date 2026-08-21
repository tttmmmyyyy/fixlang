#include <pthread.h>

// Defined by the Fix program through FFI_EXPORT. Each takes one reference to the shared value.
void fix_worker_write(void *value);
void fix_worker_drop(void *value);

// POSIX leaves barriers optional and Darwin omits them, so the meeting point the two threads need
// is built here out of a mutex and a condition variable, which every platform provides.
static pthread_mutex_t gate = PTHREAD_MUTEX_INITIALIZER;
static pthread_cond_t both_arrived = PTHREAD_COND_INITIALIZER;
static int arrived;

// Holds a thread until both threads have reached this point, so that one is copying the storage
// while the other lets go of the value. Without this the first thread finishes before the second is
// created, and the release that ends up last is never the copying thread's.
//
// Both threads pass here once, so a thread waits for the arrivals to reach two and nothing resets
// the count.
static void rendezvous(void) {
    pthread_mutex_lock(&gate);
    arrived++;
    if (arrived == 2) pthread_cond_broadcast(&both_arrived);
    while (arrived < 2) pthread_cond_wait(&both_arrived, &gate);
    pthread_mutex_unlock(&gate);
}

static void *write_thread(void *value) {
    rendezvous();
    fix_worker_write(value);
    return 0;
}

static void *drop_thread(void *value) {
    rendezvous();
    fix_worker_drop(value);
    return 0;
}

// Hands `value` to two threads, each holding one reference of its own, and waits for them.
void run_workers(void *value) {
    pthread_t writer, dropper;
    pthread_create(&writer, 0, write_thread, value);
    pthread_create(&dropper, 0, drop_thread, value);
    pthread_join(writer, 0);
    pthread_join(dropper, 0);
}
