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

int64_t fixruntime_get_number_of_processors()
{
    return (int64_t)sysconf(_SC_NPROCESSORS_ONLN);
}

void *(*ptr_fixruntime_run_function)(void *);

void *fixruntime_run_function(void *function)
{
    return (*ptr_fixruntime_run_function)(function);
}

#ifdef THREAD

typedef int *TaskFunction;
typedef int *TaskResult;
typedef struct
{
    TaskFunction function;
    TaskResult result;
    void (*release_result)(void *);
    void (*retain_result)(void *);
    pthread_mutex_t mutex;
    pthread_cond_t cond;
    uint8_t refcnt;
} Task;

// Interface functions.
void fixruntime_thread_prepare_termination();
void fixruntime_thread_terminate();
Task *fixruntime_thread_create_task(TaskFunction function, void (*release_result)(void *), void (*retain_result)(void *));
TaskResult fixruntime_thread_get_task_result(Task *task);
void fixruntime_thread_release_task(Task *task);

// Internal functions.
void fixruntime_thread_destroy_task(Task *task);
void fixruntime_thread_execute_task(Task *task);
void *fixruntime_thread_execute_task_void(void *task);

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
// void pthread_cond_signal_or_exit(pthread_cond_t *cond, const char *msg)
// {
//     if (pthread_cond_signal(cond))
//     {
//         perror(msg);
//         exit(1);
//     }
// }

void pthread_cond_broadcast_or_exit(pthread_cond_t *cond, const char *msg)
{
    if (pthread_cond_broadcast(cond))
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

#ifdef TERMINATE_TASKS
// Variables to wait all tasks to be completed.
uint64_t task_count;
pthread_mutex_t task_count_mutex;
pthread_cond_t task_count_cond;

// Initialize variables `fixruntime_thread_terminate`
void fixruntime_thread_prepare_termination()
{
    // Initialize mutex and condition variable for tasks that runs even after released on a dedicated thread.
    task_count = 0;
    if (pthread_mutex_init(&task_count_mutex, NULL))
    {
        perror("[runtime] Failed to initialize mutex `task_count_mutex`.");
        exit(1);
    }
    if (pthread_cond_init(&task_count_cond, NULL))
    {
        perror("[runtime] Failed to initialize condvar `task_count_cond`.");
        exit(1);
    }
}

// Wait for all tasks to be completed.
// This function is used only for compiler development (leak detector).
void fixruntime_thread_terminate()
{
    // Wait for all tasks that runs even after released on a dedicated thread to be completed.
    pthread_mutex_lock_or_exit(&task_count_mutex, "[runtime] Failed to lock mutex `task_count_mutex`.");
    while (task_count > 0)
    {
        pthread_cond_wait_or_exit(
            &task_count_cond,
            &task_count_mutex, "[runtime] Failed to wait condvar `task_count_cond`.");
    }
    pthread_mutex_unlock_or_exit(&task_count_mutex, "[runtime] Failed to unlock mutex `task_count_mutex`.");

    // Destroy mutex and condition variable.
    if (pthread_mutex_destroy(&task_count_mutex))
    {
        perror("[runtime] Failed to destroy mutex `task_count_mutex`.");
        exit(1);
    }
    if (pthread_cond_destroy(&task_count_cond))
    {
        perror("[runtime] Failed to destroy condvar `task_count_cond`.");
        exit(1);
    }
}

#endif // TERMINATE_TASKS

// Create an asynchronous task.
Task *fixruntime_thread_create_task(TaskFunction function, void (*release_result)(void *), void (*retain_result)(void *))
{
    Task *task = (Task *)malloc(sizeof(Task));
    task->function = function;
    task->result = NULL;
    task->release_result = release_result;
    task->retain_result = retain_result;
    task->refcnt = 2; // One ownership for this library, and one for the user.
    if (pthread_mutex_init(&task->mutex, NULL))
    {
        perror("[runtime] Failed to initialize mutex for a task.");
        exit(1);
    }
    if (pthread_cond_init(&task->cond, NULL))
    {
        perror("[runtime] Failed to initialize condition variable for a task.");
        exit(1);
    }

    // Run the task on a thread.
    pthread_t thread;
    if (pthread_create(&thread, NULL, fixruntime_thread_execute_task_void, task))
    {
        perror("[runtime] Failed to create thread to run a task.");
        exit(1);
    }
    if (pthread_detach(thread))
    {
        perror("[runtime] Failed to detach thread to run a task.");
        exit(1);
    }
#ifdef TERMINATE_TASKS
    // If the task should be terminated, then increment the counter for such tasks.
    pthread_mutex_lock_or_exit(&task_count_mutex, "[runtime] Failed to lock mutex `task_count_mutex`.");
    task_count++;
    pthread_mutex_unlock_or_exit(&task_count_mutex, "[runtime] Failed to unlock mutex `task_count_mutex`.");
#endif // TERMINATE_TASKS
    return task;
}

// Get the task result.
TaskResult fixruntime_thread_get_task_result(Task *task)
{
    pthread_mutex_lock_or_exit(&task->mutex, "[runtime] Failed to lock mutex for a task.");
    while (!task->result)
    {
        pthread_cond_wait_or_exit(&task->cond, &task->mutex, "[runtime] Failed to wait condvar for a task.");
    }
    TaskResult result = task->result;
    task->retain_result(result);
    pthread_mutex_unlock_or_exit(&task->mutex, "[runtime] Failed to unlock mutex for a task.");
    return result;
}

// Release a task.
void fixruntime_thread_release_task(Task *task)
{
    pthread_mutex_lock_or_exit(&task->mutex, "[runtime] Failed to lock mutex for a task.");
    uint8_t refcnt = --task->refcnt;
    pthread_mutex_unlock_or_exit(&task->mutex, "[runtime] Failed to unlock mutex for a task.");
    if (refcnt == 0)
    {
        fixruntime_thread_destroy_task(task);
    }
}

// Run a task on this thread.
void fixruntime_thread_execute_task(Task *task)
{
    TaskResult result = fixruntime_run_function(task->function);

    pthread_mutex_lock_or_exit(&task->mutex, "[runtime] Failed to lock mutex for a task.");
    task->result = result;
    pthread_cond_broadcast_or_exit(&task->cond, "[runtime] Failed to signal condvar for a task.");
    uint8_t refcnt = --task->refcnt;
    pthread_mutex_unlock_or_exit(&task->mutex, "[runtime] Failed to unlock mutex.");
    if (refcnt == 0)
    {
        fixruntime_thread_destroy_task(task);
    }
}

void *fixruntime_thread_execute_task_void(void *task)
{
    fixruntime_thread_execute_task((Task *)task);
    return NULL;
}

// Free the task object.
void fixruntime_thread_destroy_task(Task *task)
{
    (*task->release_result)(task->result);
    if (pthread_mutex_destroy(&task->mutex))
    {
        perror("[runtime] Failed to destroy mutex for a task.");
        exit(1);
    }
    if (pthread_cond_destroy(&task->cond))
    {
        perror("[runtime] Failed to destroy condition variable for a task.");
        exit(1);
    }

#ifdef TERMINATE_TASKS
    // If the task should run even after released on a dedicated thread, then decrement the counter for such tasks.
    pthread_mutex_lock_or_exit(&task_count_mutex, "[runtime] Failed to lock mutex `task_count_mutex`.");
    task_count--;
    pthread_cond_signal_or_exit(&task_count_cond, "[runtime] Failed to signal condvar `task_count_cond`.");
    pthread_mutex_unlock_or_exit(&task_count_mutex, "[runtime] Failed to unlock mutex `task_count_mutex`.");
#endif // TERMINATE_TASKS

    free(task);
}

typedef struct Var
{
    void *data;
    void (*release_func)(void *);
    void (*retain_func)(void *);
    pthread_mutex_t mutex;
    pthread_cond_t cond;
} Var;

Var *fixruntime_thread_var_create(void *data, void (*release_func)(void *), void (*retain_func)(void *))
{
    struct Var *handle = (struct Var *)malloc(sizeof(struct Var));

    // Create recursive mutex.
    pthread_mutexattr_t Attr;
    pthread_mutexattr_init(&Attr);
    pthread_mutexattr_settype(&Attr, PTHREAD_MUTEX_RECURSIVE);
    if (pthread_mutex_init(&handle->mutex, &Attr))
    {
        perror("[runtime] Failed to initialize mutex for a Var.");
        exit(1);
    }
    pthread_mutexattr_destroy(&Attr);

    // Create condition variable.
    if (pthread_cond_init(&handle->cond, NULL))
    {
        perror("[runtime] Failed to initialize condition variable for a Var.");
        exit(1);
    }

    // Set fields.
    handle->data = data;
    handle->release_func = release_func;
    handle->retain_func = retain_func;

    return handle;
}

void fixruntime_thread_var_destroy(Var *handle)
{
    (*handle->release_func)(handle->data);
    if (pthread_mutex_destroy(&handle->mutex))
    {
        perror("[runtime] Failed to destroy mutex for a Var.");
        exit(1);
    }
    if (pthread_cond_destroy(&handle->cond))
    {
        perror("[runtime] Failed to destroy condition variable for a Var.");
        exit(1);
    }
    free(handle);
}

void fixruntime_thread_var_lock(Var *handle)
{
    pthread_mutex_lock_or_exit(&handle->mutex, ("[runtime] Failed to lock mutex for a Var."));
}

void fixruntime_thread_var_unlock(Var *handle)
{
    pthread_mutex_unlock_or_exit(&handle->mutex, ("[runtime] Failed to unlock mutex for a Var."));
}

void fixruntime_thread_var_wait(Var *handle)
{
    pthread_cond_wait_or_exit(&handle->cond, &handle->mutex, "[runtime] Failed to wait condition variable for a Var.");
}

void fixruntime_thread_var_signalall(Var *handle)
{
    pthread_cond_broadcast_or_exit(&handle->cond, "[runtime] Failed to signal condition variable for a Var.");
}

void *fixruntime_thread_var_get(Var *handle)
{
    void *data = handle->data;
    (*handle->retain_func)(data);
    return data;
}

void fixruntime_thread_var_set(Var *handle, void *data)
{
    (*handle->release_func)(handle->data);
    handle->data = data;
}

#endif // THREAD