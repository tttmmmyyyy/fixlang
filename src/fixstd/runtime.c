/*
The C functions and values the Fix standard library is implemented with.

`fix build` compiles this source into an object file and links it into the program it builds.
*/

// glibc declares `fputs_unlocked` for a source that asks for the GNU extensions.
#define _GNU_SOURCE

#include <errno.h>
#include <inttypes.h>
#include <math.h>
#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

// Defined by the compiler, and declared in `float_text.c` as well; the two translation units carry
// the declaration because the runtime has no header of its own.
__attribute__((noreturn)) void fixruntime_abort(void);

// Print message to stderr, and flush it.
void fixruntime_eprintln(const char *msg)
{
    fprintf(stderr, "%s\n", msg);
    fflush(stderr);
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
//
// On an unbuffered `file`, such as `stderr`, the text and the newline reach the file as two writes.
// A writer the lock does not hold back -- another process writing to the same pipe, or another
// `FILE` on the same descriptor -- can put its output between the two and split the line. This is
// accepted: a fully buffered stream splits lines at the boundaries of its buffer in any case, and
// writing the line in one call costs a copy of it on every call.
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

// The value of the environment variable `name`, or NULL where it is not set. The program `fix test`
// builds from several Fix examples reads through it which example to run; a name of the runtime's
// own leaves `getenv` to the program, to declare at whatever signature it likes.
const char *fixruntime_getenv(const char *name)
{
    return getenv(name);
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
// `-170141183460469231731687303715884105728`, and the terminator follows it.
#define FIXRUNTIME_INTEGER_OPERAND_TEXT_SIZE 41

// Write the operand whose 128 bits are `high` above `low` into `buf` as the number it holds under
// `is_signed`.
//
// An operand of a narrower type arrives extended to 128 bits under its own sign, so it reads as the
// same number. A value of an unsigned type fills all its bits, so reading it as signed would report
// a number its own type cannot hold: an amount of `U64::maximum` would read as -1.
static void fixruntime_write_integer_operand(char *buf, int32_t is_signed, uint64_t low, uint64_t high)
{
    unsigned __int128 magnitude = ((unsigned __int128)high << 64) | low;
    int negative = is_signed && (high >> 63) != 0;
    if (negative)
    {
        magnitude = -magnitude;
    }
    // The digits are produced least significant first, so they are written from the end.
    char digits[FIXRUNTIME_INTEGER_OPERAND_TEXT_SIZE];
    char *first = digits + sizeof(digits);
    do
    {
        *--first = (char)('0' + (int)(magnitude % 10));
        magnitude /= 10;
    } while (magnitude != 0);
    if (negative)
    {
        *--first = '-';
    }
    size_t length = (size_t)(digits + sizeof(digits) - first);
    memcpy(buf, first, length);
    buf[length] = '\0';
}

__attribute__((noreturn)) void fixruntime_signed_overflow(const char *operation, int32_t operands_are_signed, uint64_t lhs_low, uint64_t lhs_high, uint64_t rhs_low, uint64_t rhs_high)
{
    char lhs_text[FIXRUNTIME_INTEGER_OPERAND_TEXT_SIZE];
    char rhs_text[FIXRUNTIME_INTEGER_OPERAND_TEXT_SIZE];
    fixruntime_write_integer_operand(lhs_text, operands_are_signed, lhs_low, lhs_high);
    fixruntime_write_integer_operand(rhs_text, operands_are_signed, rhs_low, rhs_high);
    fprintf(stderr, "Signed integer overflow: %s, with %s and %s\n", operation, lhs_text, rhs_text);
    fixruntime_abort();
}

__attribute__((noreturn)) void fixruntime_shift_amount_out_of_range(const char *operation, int32_t operands_are_signed, uint64_t amount_low, uint64_t amount_high)
{
    char amount_text[FIXRUNTIME_INTEGER_OPERAND_TEXT_SIZE];
    fixruntime_write_integer_operand(amount_text, operands_are_signed, amount_low, amount_high);
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