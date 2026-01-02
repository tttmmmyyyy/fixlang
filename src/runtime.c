/*
C functions / values for implementing Fix standard library.
When running program by `fix build`, then this source file will be compiled into object file and linked to the binary.
*/

#include <ctype.h>
#include <errno.h>
#include <inttypes.h>
#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#ifndef __MINGW32__
#include <sys/wait.h>
#endif // __MINGW32__
#include <unistd.h>
#include <pthread.h>

__attribute__((noreturn)) void fixruntime_abort(void);

// Print message to stderr, and flush it.
void fixruntime_eprintln(const char *msg)
{
    fprintf(stderr, "%s\n", msg);
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
FILE *fixruntime_c_stdin()
{
    return stdin;
}

FILE *fixruntime_c_stdout()
{
    return stdout;
}

FILE *fixruntime_c_stderr()
{
    return stderr;
}

int fixruntime_get_errno()
{
    return errno;
}

void fixruntime_clear_errno()
{
    errno = 0;
}

void fixruntime_index_out_of_range(int64_t idx, int64_t size)
{
    fprintf(stderr, "Index out of range: index=%" PRId64 ", size=%" PRId64 "\n", idx, size);
    fixruntime_abort();
}

#if defined(BACKTRACE)
#if defined(__linux__)
#include <backtrace.h>

static struct backtrace_state *fixruntime_backtrace_state = NULL;

// Callback for error handling in libbacktrace
static void fixruntime_backtrace_error_callback(void *data, const char *msg, int errnum)
{
    (void)data;
    fprintf(stderr, "libbacktrace error: %s (err=%d)\n", msg, errnum);
}

// Callback for each frame in backtrace
static int fixruntime_backtrace_full_callback(void *data, uintptr_t pc,
                                              const char *filename, int lineno,
                                              const char *function)
{
    int *index = (int *)data;
    fprintf(stderr, "  #%02d  %s at %s:%d (pc=0x%lx)\n",
            (*index)++, function ? function : "??",
            filename ? filename : "??", lineno,
            (unsigned long)pc);
    return 0; // 0 = continue, non-zero = stop
}

#elif defined(__APPLE__)

#include <execinfo.h>
#define MAX_BACKTRACE_FRAMES 128

#endif

#endif // BACKTRACE

// Abort function that prints backtrace if BACKTRACE is defined
__attribute__((noreturn)) void fixruntime_abort(void)
{
#if defined(BACKTRACE)
#if defined(__linux__)

    fprintf(stderr, "Backtrace:\n");

    if (!fixruntime_backtrace_state)
    {
        fixruntime_backtrace_state = backtrace_create_state(NULL, /*thread-safe=*/1,
                                                            fixruntime_backtrace_error_callback, NULL);
    }

    int frame_index = 0;
    backtrace_full(fixruntime_backtrace_state,
                   /*skip=*/1,
                   fixruntime_backtrace_full_callback,
                   fixruntime_backtrace_error_callback,
                   &frame_index);

#elif defined(__APPLE__)

    void *callstack[MAX_BACKTRACE_FRAMES];
    int frames = backtrace(callstack, MAX_BACKTRACE_FRAMES);
    fprintf(stderr, "Backtrace (%d frames):\n", frames);
    char **strs = backtrace_symbols(callstack, frames);
    if (strs)
    {
        for (int i = 1; i < frames; ++i)
        { // Skip frame 0 (current function)
            fprintf(stderr, "  #%02d  %s\n", i - 1, strs[i]);
        }
        free(strs);
    }
    else
    {
        fprintf(stderr, "Failed to get backtrace symbols\n");
    }

#endif // __linux__
#endif // BACKTRACE
    abort();
}