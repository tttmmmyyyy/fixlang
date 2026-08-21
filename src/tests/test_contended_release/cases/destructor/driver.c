#include <assert.h>
#include <pthread.h>
#include <sched.h>

// The most threads this driver hands a value to.
#define MAX_THREADS 64

// Defined by the Fix program through FFI_EXPORT: takes one reference to the round's value and drops
// it.
void fix_drop_one(void *value);

static int worker_count;
static pthread_t workers[MAX_THREADS];
static void *round_value;
static int stopping;

// Holds the workers and the thread driving the rounds, so a round starts and ends on its word.
//
// POSIX leaves barriers optional and Darwin omits them, so this is built out of a mutex and a
// condition variable, which every platform provides. It is reached twice per round, so it counts
// the rounds it has opened and a thread waits for that count to move: waiting for the arrivals to
// reach a number would let a thread that reaches the next round first mistake the arrivals of the
// round it just left for its own.
static pthread_mutex_t round_lock = PTHREAD_MUTEX_INITIALIZER;
static pthread_cond_t round_opened = PTHREAD_COND_INITIALIZER;
static int round_arrived;
static unsigned rounds_opened;

// Threads that have reached the gate, counted across every round. A round is `worker_count`
// arrivals, so a thread waits until its own group of that many is complete.
static unsigned gate_arrived;

static int destructor_runs;

void note_destructor_ran(void) { __atomic_fetch_add(&destructor_runs, 1, __ATOMIC_SEQ_CST); }

int destructor_run_count(void) { return __atomic_load_n(&destructor_runs, __ATOMIC_SEQ_CST); }

// Holds a thread until every worker and the thread driving them has reached this point.
static void rendezvous(void) {
    pthread_mutex_lock(&round_lock);
    unsigned opened = rounds_opened;
    round_arrived++;
    if (round_arrived == worker_count + 1) {
        round_arrived = 0;
        rounds_opened++;
        pthread_cond_broadcast(&round_opened);
    } else {
        while (rounds_opened == opened) pthread_cond_wait(&round_opened, &round_lock);
    }
    pthread_mutex_unlock(&round_lock);
}

// Holds a thread until every thread of the round has reached this point. They leave within a few
// hundred nanoseconds of each other, which is what puts more than one of them between reading the
// reference count of the value and decrementing it.
void wait_at_gate(void) {
    unsigned arrived = __atomic_add_fetch(&gate_arrived, 1, __ATOMIC_SEQ_CST);
    unsigned group_end = (arrived + worker_count - 1) / worker_count * worker_count;
    for (unsigned spins = 0;
         __atomic_load_n(&gate_arrived, __ATOMIC_SEQ_CST) < group_end;
         spins++) {
        // Spinning is what keeps the gate tight; yielding after a while is what keeps the round
        // finishing on a machine with fewer cores than threads.
        if (spins > (1u << 16)) sched_yield();
    }
}

static void *worker(void *unused) {
    (void)unused;
    for (;;) {
        rendezvous();
        if (stopping) return 0;
        fix_drop_one(round_value);
        rendezvous();
    }
}

// Starts the threads that take the rounds. They live across the rounds, so that a round costs the
// two words of a rendezvous rather than the creation of a thread.
void start_workers(int threads) {
    assert(threads >= 1 && threads <= MAX_THREADS);
    worker_count = threads;
    stopping = 0;
    for (int i = 0; i < threads; i++) pthread_create(&workers[i], 0, worker, 0);
}

// Hands `value` to every worker, each holding one reference of its own, and waits for the round.
void run_round(void *value) {
    round_value = value;
    rendezvous();
    rendezvous();
}

void stop_workers(void) {
    stopping = 1;
    rendezvous();
    for (int i = 0; i < worker_count; i++) pthread_join(workers[i], 0);
}
