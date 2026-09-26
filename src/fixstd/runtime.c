/*
The C functions and values the Fix standard library is implemented with.

`fix build` compiles this source into an object file and links it into the program it builds.
*/

// glibc declares `fputs_unlocked` for a source that asks for the GNU extensions.
#define _GNU_SOURCE

#include <ctype.h>
#include <errno.h>
#include <inttypes.h>
#include <math.h>
#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
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

// Each of the twelve below moves a number between a value and the bytes holding it: the
// `_to_bytes` ones write `v` into the object at `buf`, and the `_from_bytes` ones answer with the
// number the object at `buf` holds.
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
//
// Ryu carries the same table in `ryu/digit_table.h`, where it is `static`, so a source including
// that header takes a copy of it rather than sharing this one.
static const char FIXRUNTIME_DIGIT_PAIRS[201] =
    "0001020304050607080910111213141516171819202122232425262728293031323334353637383940414243444546474849"
    "5051525354555657585960616263646566676869707172737475767778798081828384858687888990919293949596979899";

// The digits a `uint64_t` takes in decimal: `18446744073709551615` is the longest.
#define FIXRUNTIME_U64_DIGITS 20

// Writes `v` at `buf` in decimal, null-terminated, and reports how many digits it took.
//
// The digits are produced from the last backwards into a scratch buffer, so that one pass writes
// them without first counting how many there are. The scratch is then copied to `buf`.
//
// Declared in `float_text.c` as well; the two translation units carry the declaration because the
// runtime has no header of its own.
int64_t fixruntime_write_u64(char *buf, uint64_t v)
{
    char digits[FIXRUNTIME_U64_DIGITS];
    int start = FIXRUNTIME_U64_DIGITS;
    while (v >= 100)
    {
        uint64_t higher = v / 100;
        unsigned int pair = (unsigned int)(v - higher * 100);
        start -= 2;
        digits[start] = FIXRUNTIME_DIGIT_PAIRS[2 * pair];
        digits[start + 1] = FIXRUNTIME_DIGIT_PAIRS[2 * pair + 1];
        v = higher;
    }
    if (v >= 10)
    {
        start -= 2;
        digits[start] = FIXRUNTIME_DIGIT_PAIRS[2 * v];
        digits[start + 1] = FIXRUNTIME_DIGIT_PAIRS[2 * v + 1];
    }
    else
    {
        digits[--start] = (char)('0' + v);
    }
    int64_t length = FIXRUNTIME_U64_DIGITS - start;
    memcpy(buf, digits + start, (size_t)length);
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
    // The magnitude is taken in unsigned arithmetic, where negating the least number an `int64_t`
    // holds is still a number. Negating it as signed overflows.
    return 1 + fixruntime_write_u64(buf + 1, -(uint64_t)v);
}

// The characters a hexadecimal digit is written with.
static const char FIXRUNTIME_HEX_DIGITS[17] = "0123456789abcdef";

// How many hexadecimal digits a pointer is written with: every digit a `uint64_t` holds.
#define FIXRUNTIME_PTR_DIGITS 16

// Writes `ptr` at `buf` as `FIXRUNTIME_PTR_DIGITS` hexadecimal digits, null-terminated, leading
// zeros among them, and reports how many digits it wrote. The pointer is taken as a `uint64_t` to
// avoid a compiler warning.
int64_t fixruntime_ptr_to_str(char *buf, uint64_t ptr)
{
    for (int i = 0; i < FIXRUNTIME_PTR_DIGITS; i++)
    {
        buf[i] = FIXRUNTIME_HEX_DIGITS[(ptr >> (4 * (FIXRUNTIME_PTR_DIGITS - 1 - i))) & 0xF];
    }
    buf[FIXRUNTIME_PTR_DIGITS] = '\0';
    return FIXRUNTIME_PTR_DIGITS;
}
// Each of the eight below writes `v` at `buf` in decimal, null-terminated, and reports how many
// bytes the text took, the null left out. A negative number is written with a `-` before its
// digits.
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

// Each of the two below reads a decimal number from the whole of `str`. The text names the number
// and nothing else: a leading space, or anything left over after the number, sets `errno` to
// `EINVAL`, and a number too large for the type sets it to `ERANGE`.
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

// The processor time the program has used so far, in the ticks C counts it in.
int64_t fixruntime_clock()
{
    return (int64_t)clock();
}

// The seconds `clocks` ticks come to.
double fixruntime_clocks_to_sec(int64_t clocks)
{
    return (double)(clock_t)clocks / CLOCKS_PER_SEC;
}

// Whether `errno` holds `EINVAL`, the error a text that does not name a number leaves.
uint8_t fixruntime_is_einval()
{
    return errno == EINVAL;
}

// Whether `errno` holds `ERANGE`, the error a number too large to hold leaves.
uint8_t fixruntime_is_erange()
{
    return errno == ERANGE;
}

// File handle resistant to being closed multiple times.
typedef struct
{
    FILE *file;
} IOHandle;

// Answers with a handle holding `file`, allocated on the heap.
IOHandle *fixruntime_iohandle_create(FILE *file)
{
    IOHandle *handle = (IOHandle *)malloc(sizeof(IOHandle));
    handle->file = file;
    return handle;
}
// Frees the handle. The file it holds is left open.
void fixruntime_iohandle_delete(IOHandle *handle)
{
    free(handle);
}
// The file the handle holds, and `NULL` once the handle has been closed.
FILE *fixruntime_iohandle_get_file(IOHandle *handle)
{
    FILE *file;
    __atomic_load(&handle->file, &file, __ATOMIC_SEQ_CST);
    return file;
}
// Takes the file out of the handle and closes it. Two threads reaching this together close the
// file once, since one of them takes it and the other finds the handle empty.
void fixruntime_iohandle_close(IOHandle *handle)
{
    FILE *file;
    FILE *closed = NULL;
    __atomic_exchange(&handle->file, &closed, &file, __ATOMIC_SEQ_CST);
    if (file)
    {
        fclose(file);
    }
}
// Each of the three below answers with one of C's standard streams. They are macros, which an FFI
// call cannot reach.
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

// Writes the null-terminated `str` followed by a newline to `file`, holding the file's lock across
// both, so no other thread's output to `file` falls between them. Returns a negative number when a
// write fails, and otherwise what `fputs` returns for `str`.
int fixruntime_fputs_line(const char *str, FILE *file)
{
    flockfile(file);
#ifdef __GLIBC__
    // Skips taking the lock that this function already holds.
    int res = fputs_unlocked(str, file);
#else
    int res = fputs(str, file);
#endif
    if (res >= 0 && putc_unlocked('\n', file) == EOF)
    {
        res = EOF;
    }
    funlockfile(file);
    return res;
}

// The value `errno` holds. `errno` is a macro, which an FFI call cannot reach.
int fixruntime_get_errno()
{
    return errno;
}

// Sets `errno` to zero.
void fixruntime_clear_errno()
{
    errno = 0;
}

// Each of the four below prints what went wrong to standard error and stops the program: an index
// fell outside its array, an array size was below zero, an array size was wider than the address
// space, or a signed operation's result did not fit its type.
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

// The bytes an operand of an integer operation takes in a report: the longest such number is
// `18446744073709551615`, and the terminator follows it.
#define FIXRUNTIME_INTEGER_OPERAND_TEXT_SIZE 21

// Write `value` into `buf` as the number it holds under `is_signed`.
//
// A value of an unsigned type fills all 64 bits, so reading it as signed would report a number its
// own type cannot hold: an amount of `U64::maximum` would read as -1.
static void fixruntime_write_integer_operand(char *buf, size_t size, int32_t is_signed, int64_t value)
{
    if (is_signed)
    {
        snprintf(buf, size, "%" PRId64, value);
    }
    else
    {
        snprintf(buf, size, "%" PRIu64, (uint64_t)value);
    }
}

__attribute__((noreturn)) void fixruntime_signed_overflow(const char *operation, int32_t operands_are_signed, int64_t lhs, int64_t rhs)
{
    char lhs_text[FIXRUNTIME_INTEGER_OPERAND_TEXT_SIZE];
    char rhs_text[FIXRUNTIME_INTEGER_OPERAND_TEXT_SIZE];
    fixruntime_write_integer_operand(lhs_text, sizeof(lhs_text), operands_are_signed, lhs);
    fixruntime_write_integer_operand(rhs_text, sizeof(rhs_text), operands_are_signed, rhs);
    fprintf(stderr, "Signed integer overflow: %s, with %s and %s\n", operation, lhs_text, rhs_text);
    fixruntime_abort();
}

__attribute__((noreturn)) void fixruntime_shift_amount_out_of_range(const char *operation, int32_t operands_are_signed, int64_t amount)
{
    char amount_text[FIXRUNTIME_INTEGER_OPERAND_TEXT_SIZE];
    fixruntime_write_integer_operand(amount_text, sizeof(amount_text), operands_are_signed, amount);
    fprintf(stderr, "Shift amount outside the width of the type: %s, with %s\n", operation, amount_text);
    fixruntime_abort();
}

__attribute__((noreturn)) void fixruntime_float_to_integer_out_of_range(const char *operation, double value)
{
    // A NaN is reported without its sign bit, which the language leaves open: the quotient that
    // produces one carries a different sign when the optimizer folds it than when the machine
    // computes it, and a report naming that bit would name the optimization level instead.
    if (isnan(value))
    {
        fprintf(stderr, "Floating-point value outside the range of the integer type: %s, with nan\n", operation);
    }
    else
    {
        fprintf(stderr, "Floating-point value outside the range of the integer type: %s, with %.17g\n", operation, value);
    }
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
    int *frame_index = (int *)data;
    fprintf(stderr, "  #%02d  %s at %s:%d (pc=0x%lx)\n",
            (*frame_index)++, function ? function : "??",
            filename ? filename : "??", lineno,
            (unsigned long)pc);
    return 0; // 0 = continue, non-zero = stop
}

#elif defined(__APPLE__)

#include <execinfo.h>
#define MAX_BACKTRACE_FRAMES 128

#endif

#endif // BACKTRACE

// Stops the program, printing the call stack it stopped at where the runtime was built with
// `BACKTRACE` defined.
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
    char **symbols = backtrace_symbols(callstack, frames);
    if (symbols)
    {
        for (int i = 1; i < frames; ++i)
        { // Skip frame 0 (current function)
            fprintf(stderr, "  #%02d  %s\n", i - 1, symbols[i]);
        }
        free(symbols);
    }
    else
    {
        fprintf(stderr, "Failed to get backtrace symbols\n");
    }

#endif // __linux__
#endif // BACKTRACE
    abort();
}