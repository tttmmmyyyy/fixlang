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

// Defined by the compiler, and declared in `float_text.c` as well; the two translation units carry
// the declaration because the runtime has no header of its own.
__attribute__((noreturn)) void fixruntime_abort(void);

// Print message to stderr, and flush it.
void fixruntime_eprintln(const char *msg)
{
    fprintf(stderr, "%s\n", msg);
    fflush(stderr);
}

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

// The two digits each number below a hundred is written with, laid end to end, so that a number is
// written two digits at a time.
static const char FIXRUNTIME_DIGIT_PAIRS[201] =
    "0001020304050607080910111213141516171819202122232425262728293031323334353637383940414243444546474849"
    "5051525354555657585960616263646566676869707172737475767778798081828384858687888990919293949596979899";

// The bytes a `uint64_t` takes in decimal: `18446744073709551615` is the longest.
#define FIXRUNTIME_U64_DIGITS 20

// Writes `v` at `buf` in decimal, null-terminated, and reports how many digits it took.
//
// The digits are built from the last of them backwards into a scratch, which is what lets one pass
// produce them without knowing first how many there are, and the scratch is then copied over.
static int64_t fixruntime_write_u64(char *buf, uint64_t v)
{
    char digits[FIXRUNTIME_U64_DIGITS];
    int at = FIXRUNTIME_U64_DIGITS;
    while (v >= 100)
    {
        uint64_t rest = v / 100;
        unsigned int pair = (unsigned int)(v - rest * 100);
        at -= 2;
        digits[at] = FIXRUNTIME_DIGIT_PAIRS[2 * pair];
        digits[at + 1] = FIXRUNTIME_DIGIT_PAIRS[2 * pair + 1];
        v = rest;
    }
    if (v >= 10)
    {
        at -= 2;
        digits[at] = FIXRUNTIME_DIGIT_PAIRS[2 * v];
        digits[at + 1] = FIXRUNTIME_DIGIT_PAIRS[2 * v + 1];
    }
    else
    {
        digits[--at] = (char)('0' + v);
    }
    int64_t length = FIXRUNTIME_U64_DIGITS - at;
    memcpy(buf, digits + at, (size_t)length);
    buf[length] = '\0';
    return length;
}

// Writes `v` at `buf` in decimal, with a sign where it is negative, and reports how many bytes it
// took, the sign among them.
static int64_t fixruntime_write_i64(char *buf, int64_t v)
{
    if (v >= 0)
    {
        return fixruntime_write_u64(buf, (uint64_t)v);
    }
    buf[0] = '-';
    // The magnitude is taken in unsigned, where negating the least number a `int64_t` holds stays a
    // number: negating it as signed would leave the type.
    return 1 + fixruntime_write_u64(buf + 1, -(uint64_t)v);
}

// The letters a hexadecimal digit is written with.
static const char FIXRUNTIME_HEX_DIGITS[17] = "0123456789abcdef";

// The hexadecimal digits a pointer is written with, which is every digit a `uint64_t` holds.
#define FIXRUNTIME_PTR_DIGITS 16

int64_t fixruntime_ptr_to_str(char *buf, uint64_t ptr) // To avoid warning, we use uint64_t instead of void*.
{
    for (int i = 0; i < FIXRUNTIME_PTR_DIGITS; i++)
    {
        buf[i] = FIXRUNTIME_HEX_DIGITS[(ptr >> (4 * (FIXRUNTIME_PTR_DIGITS - 1 - i))) & 0xF];
    }
    buf[FIXRUNTIME_PTR_DIGITS] = '\0';
    return FIXRUNTIME_PTR_DIGITS;
}
int64_t fixruntime_i8_to_str(char *buf, int8_t v)
{
    return fixruntime_write_i64(buf, v);
}
int64_t fixruntime_u8_to_str(char *buf, uint8_t v)
{
    return fixruntime_write_u64(buf, v);
}
int64_t fixruntime_i16_to_str(char *buf, int16_t v)
{
    return fixruntime_write_i64(buf, v);
}
int64_t fixruntime_u16_to_str(char *buf, uint16_t v)
{
    return fixruntime_write_u64(buf, v);
}
int64_t fixruntime_u32_to_str(char *buf, uint32_t v)
{
    return fixruntime_write_u64(buf, v);
}

int64_t fixruntime_u64_to_str(char *buf, uint64_t v)
{
    return fixruntime_write_u64(buf, v);
}

int64_t fixruntime_i32_to_str(char *buf, int32_t v)
{
    return fixruntime_write_i64(buf, v);
}

int64_t fixruntime_i64_to_str(char *buf, int64_t v)
{
    return fixruntime_write_i64(buf, v);
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

__attribute__((noreturn)) void fixruntime_index_out_of_range(int64_t idx, int64_t size)
{
    fprintf(stderr, "Index out of range: index=%" PRId64 ", size=%" PRId64 "\n", idx, size);
    fixruntime_abort();
}

__attribute__((noreturn)) void fixruntime_negative_array_size(int64_t size)
{
    fprintf(stderr, "Negative array size or capacity: %" PRId64 "\n", size);
    fixruntime_abort();
}

__attribute__((noreturn)) void fixruntime_array_size_overflow(int64_t size)
{
    fprintf(stderr, "Array size or capacity exceeds the address space: %" PRId64 "\n", size);
    fixruntime_abort();
}

__attribute__((noreturn)) void fixruntime_signed_overflow(const char *operation, int64_t lhs, int64_t rhs)
{
    fprintf(stderr, "Signed integer overflow: %s, with %" PRId64 " and %" PRId64 "\n", operation, lhs, rhs);
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