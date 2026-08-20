#include <pthread.h>

// Defined by the Fix program through FFI_EXPORT. Each takes one reference to the shared value.
void fix_worker_write(void *value);
void fix_worker_drop(void *value);

static pthread_barrier_t start;

static void *write_thread(void *value) {
    // Wait for both threads to arrive, so that one is copying the storage while the other lets go
    // of the value. Without this the first thread finishes before the second is created, and the
    // release that ends up last is never the copying thread's.
    pthread_barrier_wait(&start);
    fix_worker_write(value);
    return 0;
}

static void *drop_thread(void *value) {
    pthread_barrier_wait(&start);
    fix_worker_drop(value);
    return 0;
}

// Hands `value` to two threads, each holding one reference of its own, and waits for them.
void run_workers(void *value) {
    pthread_t writer, dropper;
    pthread_barrier_init(&start, 0, 2);
    pthread_create(&writer, 0, write_thread, value);
    pthread_create(&dropper, 0, drop_thread, value);
    pthread_join(writer, 0);
    pthread_join(dropper, 0);
    pthread_barrier_destroy(&start);
}
