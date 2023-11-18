/*
C functions / values for implementing Fix standard library.
- When running program by `fix run`, then this source file will be compiled into shared library and loaded to the JIT environment.
- When running program by `fix build`, then this source file will be compiled into object file and linked to the binary.
*/

#include <ctype.h>
#include <errno.h>
#include <inttypes.h>
#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#ifndef __MINGW32__
#include <sys/time.h>
#include <sys/wait.h>
#endif // __MINGW32__
#include <time.h>
#include <unistd.h>
#include <pthread.h>

#ifdef __MINGW32__
#define timegm _mkgmtime

struct tm *localtime_r(const time_t *timer, struct tm *buf)
{
    if (localtime_s(buf, timer))
    {
        return NULL;
    }
    return buf;
}

struct tm *gmtime_r(const time_t *timer, struct tm *buf)
{
    if (gmtime_s(buf, timer))
    {
        return NULL;
    }
    return buf;
}
#endif // __MINGW32__

// Print message to stderr, and flush it.
void fixruntime_eprint(const char *msg)
{
    fprintf(stderr, "%s", msg);
    fflush(stderr);
}

// NOTE: Maybe should we define following functions by LLVM to better optimization opportunity?
void fixruntime_u8_to_bytes(uint8_t *buf, uint8_t v)
{
    *buf = v;
}
void fixruntime_u16_to_bytes(uint16_t *buf, uint16_t v)
{
    *buf = v;
}
void fixruntime_u32_to_bytes(uint32_t *buf, uint32_t v)
{
    *buf = v;
}
void fixruntime_u64_to_bytes(uint64_t *buf, uint64_t v)
{
    *buf = v;
}
void fixruntime_f32_to_bytes(float *buf, float v)
{
    *buf = v;
}
void fixruntime_f64_to_bytes(double *buf, double v)
{
    *buf = v;
}
uint8_t fixruntime_u8_from_bytes(uint8_t *buf)
{
    return *buf;
}
uint16_t fixruntime_u16_from_bytes(uint16_t *buf)
{
    return *buf;
}
uint32_t fixruntime_u32_from_bytes(uint32_t *buf)
{
    return *buf;
}
uint64_t fixruntime_u64_from_bytes(uint64_t *buf)
{
    return *buf;
}
float fixruntime_f32_from_bytes(float *buf)
{
    return *buf;
}
double fixruntime_f64_from_bytes(double *buf)
{
    return *buf;
}

void fixruntime_ptr_to_str(char *buf, uint64_t ptr) // To avoid warning, we use uint64_t instead of void*.
{
    sprintf(buf, "%016" PRIx64, ptr);
}
void fixruntime_i8_to_str(char *buf, int8_t v)
{
    sprintf(buf, "%" PRId8, v);
}
void fixruntime_u8_to_str(char *buf, uint8_t v)
{
    sprintf(buf, "%" PRIu8, v);
}
void fixruntime_i16_to_str(char *buf, int16_t v)
{
    sprintf(buf, "%" PRId16, v);
}
void fixruntime_u16_to_str(char *buf, uint16_t v)
{
    sprintf(buf, "%" PRIu16, v);
}
void fixruntime_u32_to_str(char *buf, uint32_t v)
{
    sprintf(buf, "%" PRIu32, v);
}

void fixruntime_u64_to_str(char *buf, uint64_t v)
{
    sprintf(buf, "%" PRIu64, v);
}

void fixruntime_i32_to_str(char *buf, int32_t v)
{
    sprintf(buf, "%" PRId32, v);
}

void fixruntime_i64_to_str(char *buf, int64_t v)
{
    sprintf(buf, "%" PRId64, v);
}

void fixruntime_f32_to_str(char *buf, float v)
{
    sprintf(buf, "%f", v);
}

void fixruntime_f32_to_str_exp(char *buf, float v)
{
    sprintf(buf, "%e", v);
}

void fixruntime_f32_to_str_exp_precision(char *buf, float v, uint8_t precision)
{
    char specifier[7]; // len(%.255e) + 1
    sprintf(specifier, "%%.%" PRIu8 "e", precision);
    sprintf(buf, specifier, v);
}

void fixruntime_f32_to_str_precision(char *buf, float v, uint8_t precision)
{
    char specifier[7]; // len(%.255f) + 1
    sprintf(specifier, "%%.%" PRIu8 "f", precision);
    sprintf(buf, specifier, v);
}

void fixruntime_f64_to_str(char *buf, double v)
{
    sprintf(buf, "%lf", v);
}

void fixruntime_f64_to_str_exp(char *buf, double v)
{
    sprintf(buf, "%le", v);
}

void fixruntime_f64_to_str_exp_precision(char *buf, double v, uint8_t precision)
{
    char specifier[8]; // len(%.255le) + 1
    sprintf(specifier, "%%.%" PRIu8 "le", precision);
    sprintf(buf, specifier, v);
}

void fixruntime_f64_to_str_precision(char *buf, double v, uint8_t precision)
{
    char specifier[8]; // len(%.255lf) + 1
    sprintf(specifier, "%%.%" PRIu8 "lf", precision);
    sprintf(buf, specifier, v);
}

int64_t fixruntime_strtoll_10(const char *str)
{
    char *endptr;
    errno = 0;
    if (isspace(*str))
    {
        errno = EINVAL;
        return (int64_t)0;
    }
    int64_t v = (int64_t)strtoll(str, &endptr, 10);
    if (endptr == str || *endptr != '\0')
    {
        errno = EINVAL;
    }
    return v;
}

uint64_t fixruntime_strtoull_10(const char *str)
{
    char *endptr;
    errno = 0;
    if (isspace(*str))
    {
        errno = EINVAL;
        return (int64_t)0;
    }
    uint64_t v = (uint64_t)strtoull(str, &endptr, 10);
    if (endptr == str || *endptr != '\0')
    {
        errno = EINVAL;
    }
    return v;
}

double fixruntime_strtod(const char *str)
{
    char *endptr;
    errno = 0;
    if (isspace(*str))
    {
        errno = EINVAL;
        return (int64_t)0;
    }
    double v = strtod(str, &endptr);
    if (endptr == str || *endptr != '\0')
    {
        errno = EINVAL;
    }
    return v;
}

float fixruntime_strtof(const char *str)
{
    char *endptr;
    errno = 0;
    if (isspace(*str))
    {
        errno = EINVAL;
        return (int64_t)0;
    }
    float v = strtof(str, &endptr);
    if (endptr == str || *endptr != '\0')
    {
        errno = EINVAL;
    }
    return v;
}

void fixruntime_clock_gettime(int64_t *ret)
{
    struct timespec ts;
    clock_gettime(CLOCK_REALTIME, &ts);
    ret[0] = (int64_t)ts.tv_sec;
    ret[1] = (int64_t)ts.tv_nsec;
}
void fixruntime_gmlocaltime(uint8_t is_local, uint64_t sec, int64_t *ret)
{
    // struct tm *gmtime_r(const time_t *timep, struct tm *result);
    time_t time = (time_t)sec;
    struct tm datetime;
    struct tm *is_suc;
    if (is_local > 0)
    {
        is_suc = localtime_r(&time, &datetime);
    }
    else
    {
        is_suc = gmtime_r(&time, &datetime);
    }
    ret[0] = (int64_t)datetime.tm_sec;
    ret[1] = (int64_t)datetime.tm_min;
    ret[2] = (int64_t)datetime.tm_hour;
    ret[3] = (int64_t)datetime.tm_mday;
    ret[4] = (int64_t)datetime.tm_mon;
    ret[5] = (int64_t)datetime.tm_year;
    ret[6] = (int64_t)datetime.tm_wday;
    ret[7] = (int64_t)datetime.tm_yday;
    ret[8] = (int64_t)datetime.tm_isdst;
    ret[9] = (int64_t)(is_suc == NULL);
}
int64_t fixruntime_timegmlocal(uint8_t is_local, int64_t *data)
{
    struct tm datetime;
    datetime.tm_sec = (int)data[0];
    datetime.tm_min = (int)data[1];
    datetime.tm_hour = (int)data[2];
    datetime.tm_mday = (int)data[3];
    datetime.tm_mon = (int)data[4];
    datetime.tm_year = (int)data[5];
    datetime.tm_wday = (int)data[6];
    datetime.tm_yday = (int)data[7];
    datetime.tm_isdst = (int)data[8];
    time_t ret;
    if (is_local > 0)
    {
        ret = mktime(&datetime);
    }
    else
    {
        ret = timegm(&datetime);
    }
    return (int64_t)ret;
}

int64_t fixruntime_clock()
{
    return (int64_t)clock();
}

double fixruntime_clocks_to_sec(int64_t clocks)
{
    return (double)(clock_t)clocks / CLOCKS_PER_SEC;
}

uint8_t fixruntime_is_einval()
{
    return errno == EINVAL;
}

uint8_t fixruntime_is_erange()
{
    return errno == ERANGE;
}

// Fork child process and launch process by execvp.
// * `error_buf` - If no error occurrs, error_buf will be set to pointing NULL.
//                 Otherwise, error_buf will be set to pointing to null-terminated error string. In this case, the caller should free the string buffer.
// * `streams` - If succceeds, streams[0], streams[1] and streams[2] are set FILE handles that are piped to stdio, stdout and stderr of child process.
void fixruntime_fork_execvp(const char *program_path, char *const argv[], char **out_error, FILE *out_streams[], int64_t *out_pid)
{
    *out_error = NULL;

    int pipes[3][2]; // in, out, err

    for (int i = 0; i < 3; i++)
    {
        if (pipe(pipes[i]))
        {
            // Failed creating pipes.
            for (int j = 0; j < i; j++)
            {
                close(pipes[j][0]);
                close(pipes[j][1]);
            }

            const char *msg = "Failed to create pipe.";
            *out_error = (char *)malloc(sizeof(char) * (strlen(msg) + 1));
            strcpy(*out_error, msg);

            return;
        }
    }
    pid_t pid = fork();
    if (!pid)
    {
        // In child process,

        dup2(pipes[0][0], 0); // stdin
        dup2(pipes[1][1], 1); // stdout
        dup2(pipes[2][1], 2); // stderr

        for (int i = 0; i < 3; i++)
        {
            close(pipes[i][0]);
            close(pipes[i][1]);
        }

        execvp(program_path, argv);

        // If execvp fails,
        exit(1);
    }
    else
    { // In parent process,
        if (pid < 0)
        {
            // Failed creating process.

            const char *msg = "Failed to create child process.";
            *out_error = (char *)malloc(sizeof(char) * (strlen(msg) + 1));
            strcpy(*out_error, msg);

            return;
        }
        close(pipes[0][0]);
        close(pipes[1][1]);
        close(pipes[2][1]);

        out_streams[0] = fdopen(pipes[0][1], "w");
        out_streams[1] = fdopen(pipes[1][0], "r");
        out_streams[2] = fdopen(pipes[2][0], "r");

        *out_pid = (int64_t)pid;

        return;
    }
}

// Wait termination of child process specified.
// * `timeout` - Positive for timeout value (in seconds), negative for no timeout.
// * `out_is_timeout` - Set to 1 when return by timeout, or set to 0 otherwise. Should not be NULL when `timeout` is not NULL.
// * `out_wait_failed` - Set to 1 when waiting child process failed, or set to 0 otherwise.
// * `out_exit_status` - The exit status of child process is stored to the address specified this argument. This value should be used only when `*out_exit_status_available == 1`.
// * `out_exit_status_available` - Set to 1 when exit status is available, or set to 0 otherwise.
// * `out_stop_signal` - The signal number which caused the termination of the child process. This value should be used only when `*out_stop_signal_available == 1`.
// * `out_stop_signal_available` - Set to 1 when the stop signal number is available, or set to 0 otherwise.
void fixruntime_wait_subprocess(int64_t pid, double timeout,
                                uint8_t *out_is_timeout,
                                uint8_t *out_wait_failed,
                                uint8_t *out_exit_status,
                                uint8_t *out_exit_status_available,
                                uint8_t *out_stop_signal,
                                uint8_t *out_stop_signal_available)
{
    int wait_status;
    pid_t wait_return;
    struct timespec start;
    double start_f;
    struct timespec now;
    double now_f;

    *out_is_timeout = 0;
    *out_exit_status_available = 0;
    *out_stop_signal_available = 0;
    *out_wait_failed = 0;

    if (timeout < 0.0)
    {
        wait_return = waitpid((pid_t)pid, &wait_status, 0);
    }
    else
    {
        clock_gettime(CLOCK_MONOTONIC, &start);
        start_f = (double)start.tv_sec + (double)start.tv_nsec / 1e9;
        while (1)
        {
            // TODO: fix busy wait (using threads?)
            wait_return = waitpid((pid_t)pid, &wait_status, WNOHANG);
            if (wait_return != 0)
            {
                break;
            }
            clock_gettime(CLOCK_MONOTONIC, &now);
            now_f = (double)now.tv_sec + (double)now.tv_nsec / 1e9;
            if (now_f - start_f >= timeout)
            {
                *out_is_timeout = 1;
                break;
            }
        }
    }
    if (wait_return == -1)
    {
        *out_wait_failed = 1;
        return;
    }
    if (WIFEXITED(wait_status))
    {
        *out_exit_status_available = 1;
        *out_exit_status = (uint8_t)WEXITSTATUS(wait_status);
        return;
    }
    if (WIFSIGNALED(wait_status))
    {
        *out_stop_signal_available = 1;
        *out_stop_signal = (uint8_t)WSTOPSIG(wait_status);
        return;
    }
}

// File handle resistant to being closed multiple times.
typedef struct
{
    FILE *file;
} IOHandle;

IOHandle *fixruntime_iohandle_create(FILE *file)
{
    IOHandle *handle = (IOHandle *)malloc(sizeof(IOHandle));
    handle->file = file;
    return handle;
}
void fixruntime_iohandle_delete(IOHandle *handle)
{
    free(handle);
}
FILE *fixruntime_iohandle_get_file(IOHandle *handle)
{
    FILE *file;
    __atomic_load(&handle->file, &file, __ATOMIC_SEQ_CST);
    return file;
}
void fixruntime_iohandle_close(IOHandle *handle)
{
    FILE *file;
    FILE *new_val = NULL;
    __atomic_exchange(&handle->file, &new_val, &file, __ATOMIC_SEQ_CST);
    if (file)
    {
        fclose(file);
    }
}

/*
Thread pool implementation.
- Call `fixruntime_threadpool_initialize` to initialize thread pool.
- Create future object by `fixruntime_threadpool_create_future`.
    - This function takes `TaskFunc` and `TaskData` as arguments, and call `TaskFunc` with `TaskData` when the task is executed.
- Wait for future to be completed by `fixruntime_threadpool_wait_future`.
- Delete future by `fixruntime_threadpool_delete_future`.
- Get `TaskData` object from `Future` object by `fixruntime_threadpool_get_task_data`.
A future must be deleted exactly once. A future must not be deleted while another thread is waiting it.
*/

typedef int *TaskData;
typedef void (*TaskFunc)(TaskData);
typedef struct IFuture
{
    TaskFunc func;
    TaskData data;
    struct IFuture *next; // A pointer to the next future in the queue.
    pthread_mutex_t mutex;
    pthread_cond_t cond;
    uint8_t status;
    uint8_t refcnt;
} Future;

// Interface functions.
void fixruntime_threadpool_initialize();
Future *fixruntime_threadpool_create_future(TaskFunc func, TaskData data);
void fixruntime_threadpool_wait_future(Future *future);
void fixruntime_threadpool_delete_future(Future *future);
TaskData fixruntime_threadpool_get_task_data(Future *future);

// Internal functions.
void *fixruntime_threadpool_on_thread(void *);
void fixruntime_threadpool_push_future(Future *future);
Future *fixruntime_threadpool_pop_future();
void fixruntime_threadpool_release_future(Future *future);

// Status of a future.
// The status transits as WAITING -> RUNNING -> COMPLETED, and any status can transit to CANCELLED.
// If a future is cancelled before it is completed, then it is marked as CANCELLED and will be deleted by the thread that runs it or waits it.
// If a future is cancelled after it is completed, then it is deleted immediately.
uint8_t FUTURE_STATUS_WAITING = 0;
uint8_t FUTURE_STATUS_RUNNING = 1;
uint8_t FUTURE_STATUS_COMPLETED = 2;

// Future queue.
Future *future_queue_first = NULL;
Future *future_queue_last = NULL;
pthread_mutex_t future_queue_mutex;
pthread_cond_t future_queue_cond;

// Thread pool.
pthread_t *thread_pool;

// Utility functions.
void pthread_mutex_lock_or_exit(pthread_mutex_t *mutex, const char *msg)
{
    if (pthread_mutex_lock(mutex))
    {
        perror(msg);
        exit(1);
    }
}
void pthread_mutex_unlock_or_exit(pthread_mutex_t *mutex, const char *msg)
{
    if (pthread_mutex_unlock(mutex))
    {
        perror(msg);
        exit(1);
    }
}
void pthread_cond_wait_or_exit(pthread_cond_t *cond, pthread_mutex_t *mutex, const char *msg)
{
    if (pthread_cond_wait(cond, mutex))
    {
        perror(msg);
        exit(1);
    }
}
void pthread_cond_signal_or_exit(pthread_cond_t *cond, const char *msg)
{
    if (pthread_cond_signal(cond))
    {
        perror(msg);
        exit(1);
    }
}

// Initialize thread pool.
void fixruntime_threadpool_initialize()
{
    // Initialize mutex for future queue.
    if (pthread_mutex_init(&future_queue_mutex, NULL))
    {
        perror("[runtime] Failed to initialize mutex for future queue.");
        exit(1);
    }
    // Initialize condition variable for future queue.
    if (pthread_cond_init(&future_queue_cond, NULL))
    {
        perror("[runtime] Failed to initialize condition variable for future queue.");
        exit(1);
    }
    // Initialize threads.
    // https://stackoverflow.com/questions/150355/programmatically-find-the-number-of-cores-on-a-machine
    int num_cpu = sysconf(_SC_NPROCESSORS_ONLN);
    thread_pool = (pthread_t *)malloc(sizeof(pthread_t) * num_cpu);
    for (int i = 0; i < num_cpu; i++)
    {
        if (pthread_create(&thread_pool[i], NULL, fixruntime_threadpool_on_thread, NULL))
        {
            perror("[runtime] Failed to create thread.");
            exit(1);
        }
    }
}

// Push a future to the queue.
void fixruntime_threadpool_push_future(Future *future)
{
    pthread_mutex_lock_or_exit(&future_queue_mutex, "[runtime] Failed to lock mutex.");
    if (future_queue_last)
    {
        future_queue_last->next = future;
    }
    else
    {
        future_queue_first = future;
    }
    future_queue_last = future;
    pthread_cond_signal_or_exit(&future_queue_cond, "[runtime] Failed to signal condition variable.");
    pthread_mutex_unlock_or_exit(&future_queue_mutex, "[runtime] Failed to unlock mutex.");
}

// Pop a future from the queue.
// If the queue is empty, then wait for a future to be pushed.
Future *fixruntime_threadpool_pop_future()
{
    pthread_mutex_lock_or_exit(&future_queue_mutex, "[runtime] Failed to lock mutex.");
    while (!future_queue_first) // Wait for a future to be pushed.
    {
        pthread_cond_wait_or_exit(&future_queue_cond, &future_queue_mutex, "[runtime] Failed to wait condition variable.");
    }
    Future *future = future_queue_first;
    future_queue_first = future->next;
    if (!future_queue_first)
    {
        future_queue_last = NULL;
    }
    pthread_mutex_unlock_or_exit(&future_queue_mutex, "[runtime] Failed to unlock mutex.");
    return future;
}

// Create a future and push it to the queue.
Future *fixruntime_threadpool_create_future(TaskFunc func, TaskData data)
{
    Future *future = (Future *)malloc(sizeof(Future));
    future->func = func;
    future->data = data;
    future->next = NULL;
    future->status = FUTURE_STATUS_WAITING;
    future->refcnt = 2; // One ownership for this library, and one for the user.
    if (pthread_mutex_init(&future->mutex, NULL))
    {
        perror("[runtime] Failed to initialize mutex for a future.");
        exit(1);
    }
    if (pthread_cond_init(&future->cond, NULL))
    {
        perror("[runtime] Failed to initialize condition variable for a future.");
        exit(1);
    }
    fixruntime_threadpool_push_future(future);
    return future;
}

// Wait for the future to be completed.
void fixruntime_threadpool_wait_future(Future *future)
{
    pthread_mutex_lock_or_exit(&future->mutex, "[runtime] Failed to lock mutex.");
    if (future->status == FUTURE_STATUS_WAITING)
    {
        // If the future is still waiting, then run it on this thread.
        future->status = FUTURE_STATUS_RUNNING;
        pthread_mutex_unlock_or_exit(&future->mutex, "[runtime] Failed to unlock mutex.");
        future->func(future->data);
        pthread_mutex_lock_or_exit(&future->mutex, "[runtime] Failed to lock mutex.");
        future->status = FUTURE_STATUS_COMPLETED;
        pthread_cond_signal_or_exit(&future->cond, "[runtime] Failed to signal condition variable.");
        pthread_mutex_unlock_or_exit(&future->mutex, "[runtime] Failed to unlock mutex.");
    }
    else if (future->status == FUTURE_STATUS_RUNNING)
    {
        // Wait for the future to be completed.
        while (future->status == FUTURE_STATUS_RUNNING)
        {
            pthread_cond_wait_or_exit(&future->cond, &future->mutex, "[runtime] Failed to wait condition variable.");
        }
        pthread_mutex_unlock_or_exit(&future->mutex, "[runtime] Failed to unlock mutex.");
    }
    else
    {
        // If the task is already completed, then do nothing.
        pthread_mutex_unlock_or_exit(&future->mutex, "[runtime] Failed to unlock mutex.");
    }
}

// Delete a future.
void fixruntime_threadpool_delete_future(Future *future)
{
    pthread_mutex_lock_or_exit(&future->mutex, "[runtime] Failed to lock mutex.");
    future->status = FUTURE_STATUS_COMPLETED;
    uint8_t refcnt = --future->refcnt;
    pthread_mutex_unlock_or_exit(&future->mutex, "[runtime] Failed to unlock mutex.");
    if (refcnt == 0)
    {
        fixruntime_threadpool_release_future(future);
    }
}

// Get the task data from the future.
TaskData fixruntime_threadpool_get_task_data(Future *future)
{
    return future->data;
}

// Run each future on a thread.
void *fixruntime_threadpool_on_thread(void *data)
{
    while (1)
    {
        Future *future = fixruntime_threadpool_pop_future();
        pthread_mutex_lock_or_exit(&future->mutex, "[runtime] Failed to lock mutex.");
        if (future->status == FUTURE_STATUS_COMPLETED || future->status == FUTURE_STATUS_RUNNING)
        {
            // The task is already completed or running in another thread, then do nothing.
            uint8_t refcnt = --future->refcnt;
            pthread_mutex_unlock_or_exit(&future->mutex, "[runtime] Failed to unlock mutex.");
            if (refcnt == 0)
            {
                fixruntime_threadpool_release_future(future);
            }
            continue;
        }
        future->status = FUTURE_STATUS_RUNNING;
        pthread_mutex_unlock_or_exit(&future->mutex, "[runtime] Failed to unlock mutex.");
        future->func(future->data);
        pthread_mutex_lock_or_exit(&future->mutex, "[runtime] Failed to lock mutex.");
        future->status = FUTURE_STATUS_COMPLETED;
        uint8_t refcnt = --future->refcnt;
        pthread_cond_signal_or_exit(&future->cond, "[runtime] Failed to signal condition variable.");
        pthread_mutex_unlock_or_exit(&future->mutex, "[runtime] Failed to unlock mutex.");
        if (refcnt == 0)
        {
            fixruntime_threadpool_release_future(future);
        }
    }
}

// Delete the future.
void fixruntime_threadpool_release_future(Future *future)
{
    if (pthread_mutex_destroy(&future->mutex))
    {
        perror("[runtime] Failed to destroy mutex for a future.");
        exit(1);
    }
    if (pthread_cond_destroy(&future->cond))
    {
        perror("[runtime] Failed to destroy condition variable for a future.");
        exit(1);
    }
    free(future);
}