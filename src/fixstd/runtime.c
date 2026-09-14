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
#include "ryu/ryu.h"

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

// Stops the program unless the text `snprintf` reported fits `size`.
//
// The caller in `src/fixstd/std.fix` derives that buffer's size from the widest text the format
// can write, so a text that does not fit means the derivation is wrong. Stopping here names the
// two sizes, where letting the write run on would leave the heap damaged and the program going.
//
// # Arguments
// * `written` - What `snprintf` answered: the length of the text, without its null, or a negative
//   number where it could not write the text at all.
// * `size` - The bytes the buffer holds.
static void fixruntime_check_float_text(int written, int64_t size)
{
    if (written < 0)
    {
        fprintf(stderr, "Writing a number as text failed\n");
        fixruntime_abort();
    }
    if ((int64_t)written + 1 > size)
    {
        fprintf(stderr, "A number's text takes %" PRId64 " bytes and its buffer holds %" PRId64 "\n",
                (int64_t)written + 1, size);
        fixruntime_abort();
    }
}

void fixruntime_f32_to_str_exp_precision(char *buf, int64_t size, float v, uint8_t precision)
{
    fixruntime_check_float_text(snprintf(buf, (size_t)size, "%.*e", (int)precision, v), size);
}

void fixruntime_f32_to_str_precision(char *buf, int64_t size, float v, uint8_t precision)
{
    fixruntime_check_float_text(snprintf(buf, (size_t)size, "%.*f", (int)precision, v), size);
}

void fixruntime_f64_to_str_exp_precision(char *buf, int64_t size, double v, uint8_t precision)
{
    fixruntime_check_float_text(snprintf(buf, (size_t)size, "%.*le", (int)precision, v), size);
}

void fixruntime_f64_to_str_precision(char *buf, int64_t size, double v, uint8_t precision)
{
    fixruntime_check_float_text(snprintf(buf, (size_t)size, "%.*lf", (int)precision, v), size);
}

// Writes the scientific text Ryu produced the way Fix spells a floating point number.
//
// `sci` holds what `d2s_buffered_n` or `f2s_buffered_n` wrote: a sign, the shortest digits that
// read back as the number with a point after the first of them, `E`, and the power of ten those
// digits are multiplied by. Fix writes those digits positionally where the point falls inside or
// near them, and as a power of ten otherwise, so that the text stays about as wide as the digits
// it carries: `1e300` rather than a 1 followed by 300 zeros.
//
// # Arguments
// * `sci` - The scientific text, null-terminated.
// * `buf` - Where the text is written, null-terminated.
// * `size` - The bytes `buf` holds.
// * `positional_low`, `positional_high` - The window the point is written positionally in:
//   `positional_low < point <= positional_high`, where `10^(point-1) <= |v| < 10^point`. A wider
//   window costs zeros, so it is drawn around the digits the type carries.
static void fixruntime_float_shortest_to_str(const char *sci, char *buf, int64_t size,
                                             int positional_low, int positional_high)
{
    int read = 0;
    int negative = sci[0] == '-';
    if (negative)
    {
        read = 1;
    }

    // Ryu writes `Infinity` and `NaN` where the number is not finite. Fix writes `inf` and `nan`,
    // which is what `Std::FromString` takes back.
    if (sci[read] < '0' || sci[read] > '9')
    {
        const char *special = sci[read] == 'N' ? "nan" : (negative ? "-inf" : "inf");
        int length = (int)strlen(special);
        fixruntime_check_float_text(length, size);
        memcpy(buf, special, (size_t)length + 1);
        return;
    }

    char digits[32];
    int digit_count = 0;
    for (; sci[read] != 'E'; read++)
    {
        if (sci[read] != '.')
        {
            digits[digit_count++] = sci[read];
        }
    }
    // Where the point falls among the digits: `10^(point-1) <= |v| < 10^point`.
    int point = (int)strtol(&sci[read + 1], NULL, 10) + 1;
    // The power of ten the digits, read as one whole number, are multiplied by.
    int scale = point - digit_count;

    // The widest text this writes is a sign, the digits, a point and a 4 byte power of ten, which
    // an `F64`'s 17 digits make 24 bytes of, and the null makes 25.
    char text[32];
    int written = 0;
    if (negative)
    {
        text[written++] = '-';
    }
    if (point > positional_low && point <= positional_high)
    {
        if (scale >= 0)
        {
            // 1234e3 -> 1234000.0
            memcpy(text + written, digits, (size_t)digit_count);
            written += digit_count;
            for (int i = 0; i < scale; i++)
            {
                text[written++] = '0';
            }
            text[written++] = '.';
            text[written++] = '0';
        }
        else if (point > 0)
        {
            // 1234e-2 -> 12.34
            memcpy(text + written, digits, (size_t)point);
            written += point;
            text[written++] = '.';
            memcpy(text + written, digits + point, (size_t)(digit_count - point));
            written += digit_count - point;
        }
        else
        {
            // 1234e-6 -> 0.001234
            text[written++] = '0';
            text[written++] = '.';
            for (int i = 0; i < -point; i++)
            {
                text[written++] = '0';
            }
            memcpy(text + written, digits, (size_t)digit_count);
            written += digit_count;
        }
    }
    else
    {
        // 1234e30 -> 1.234e33
        text[written++] = digits[0];
        if (digit_count > 1)
        {
            text[written++] = '.';
            memcpy(text + written, digits + 1, (size_t)(digit_count - 1));
            written += digit_count - 1;
        }
        text[written++] = 'e';
        int exponent = point - 1;
        if (exponent < 0)
        {
            text[written++] = '-';
            exponent = -exponent;
        }
        char reversed[8];
        int length = 0;
        do
        {
            reversed[length++] = (char)('0' + exponent % 10);
            exponent /= 10;
        } while (exponent != 0);
        while (length > 0)
        {
            text[written++] = reversed[--length];
        }
    }
    text[written] = '\0';

    fixruntime_check_float_text(written, size);
    memcpy(buf, text, (size_t)written + 1);
}

void fixruntime_f32_to_str_shortest(char *buf, int64_t size, float v)
{
    char sci[32];
    sci[f2s_buffered_n(v, sci)] = '\0';
    fixruntime_float_shortest_to_str(sci, buf, size, -6, 13);
}

void fixruntime_f64_to_str_shortest(char *buf, int64_t size, double v)
{
    char sci[32];
    sci[d2s_buffered_n(v, sci)] = '\0';
    fixruntime_float_shortest_to_str(sci, buf, size, -5, 16);
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