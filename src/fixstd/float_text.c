/*
Writing a floating point number as text and reading one back, for `Std::F64` and `Std::F32`.

The shortest text that reads back as the number is found by Ryu, whose sources sit beside this one
under `ryu/`. Reading goes through C's `strtod`, under a locale of this file's own so that the
point is the character Ryu writes whatever locale the program runs in.

This file is compiled with optimization where the rest of the runtime is not: what it does —
Ryu's search, and the placing of the digits it answers with — is the runtime's one piece of
arithmetic rather than a call into C's library.
*/

// `strtod_l` and `newlocale` are what read a number under a locale of our own choosing. glibc
// declares them for a source that asks for the GNU extensions, and macOS in a header of its own.
#define _GNU_SOURCE

#include <ctype.h>
#include <errno.h>
#include <inttypes.h>
#include <locale.h>
#include <math.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#ifdef __APPLE__
#include <xlocale.h>
#endif
#include "ryu/ryu.h"

// Defined by the compiler, and declared in `runtime.c` as well; the two translation units carry
// the declaration because the runtime has no header of its own.
__attribute__((noreturn)) void fixruntime_abort(void);

// Writes `v` at `buf` in decimal, null-terminated, and reports how many digits it took. Defined
// in `runtime.c`, and declared here because the runtime has no header of its own.
int64_t fixruntime_write_u64(char *buf, uint64_t v);

// The bytes `fixruntime_write_float_text` builds a text in. The static assertions at the two
// entry points hold each window to what fits here, since a wider one would write past it.
#define FLOAT_TEXT_SIZE 48

// The bytes a window asks for at its widest: a sign, a point, the zeros either edge of the window
// allows beside the digits, the digits themselves, a power of ten of up to four bytes, and a null.
#define WIDEST_FLOAT_TEXT_SIZE(low, high, digits) \
    (1 + 2 + ((-(low)) > (high) ? (-(low)) : (high)) + (digits) + 4 + 1)

// The window the point is written positionally in, and the digits the type takes at its widest.
#define F32_POSITIONAL_LOW (-6)
#define F32_POSITIONAL_HIGH 13
#define F32_DIGITS 9
#define F64_POSITIONAL_LOW (-5)
#define F64_POSITIONAL_HIGH 16
#define F64_DIGITS 17

// Answers with `written` where a text of that many bytes, and the null after it, fit `size`, and
// stops the program where they do not.
//
// The caller in `src/fixstd/std.fix` derives that buffer's size from the widest text it can be
// asked for, so a text that does not fit means the derivation is wrong. Stopping here names the
// two sizes, where letting the write run on would leave the heap damaged and the program going.
//
// # Arguments
// * `written` - The length of the text, without its null. A negative number is what `snprintf`
//   answers where it could not write the text at all.
// * `size` - The bytes the buffer holds.
static int64_t fixruntime_check_float_text(int written, int64_t size)
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
    return written;
}

// Each of the four below writes `v` at `buf` with `precision` digits after the point, null-
// terminated, and reports how many bytes the text took, the null left out. The `exp` ones write
// the number in scientific notation, and the others write it positionally.
int64_t fixruntime_f32_to_str_exp_precision(char *buf, int64_t size, float v, uint8_t precision)
{
    return fixruntime_check_float_text(snprintf(buf, (size_t)size, "%.*e", (int)precision, v), size);
}

int64_t fixruntime_f32_to_str_precision(char *buf, int64_t size, float v, uint8_t precision)
{
    return fixruntime_check_float_text(snprintf(buf, (size_t)size, "%.*f", (int)precision, v), size);
}

int64_t fixruntime_f64_to_str_exp_precision(char *buf, int64_t size, double v, uint8_t precision)
{
    return fixruntime_check_float_text(snprintf(buf, (size_t)size, "%.*le", (int)precision, v), size);
}

int64_t fixruntime_f64_to_str_precision(char *buf, int64_t size, double v, uint8_t precision)
{
    return fixruntime_check_float_text(snprintf(buf, (size_t)size, "%.*lf", (int)precision, v), size);
}

// Writes the scientific text Ryu produced the way Fix spells a floating point number, and
// reports how many bytes the text took.
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
//   window costs zeros, so it is drawn around the digits the type carries, and the widest text it
//   allows is what sizes the buffer `to_string` passes in — must stay in sync with the `size` the
//   `ToString` implementations in `src/fixstd/std.fix` derive.
static int64_t fixruntime_write_float_text(const char *sci, char *buf, int64_t size,
                                                int positional_low, int positional_high)
{
    int read = 0;
    int negative = sci[0] == '-';
    if (negative)
    {
        read = 1;
    }

    // Ryu writes `Infinity` and `NaN` where the number is not finite, and digits everywhere else,
    // zero included, which it writes as `0E0`. Fix writes `inf` and `nan`, which is what
    // `Std::FromString` takes back.
    if (sci[read] < '0' || sci[read] > '9')
    {
        const char *special;
        if (sci[read] == 'N')
        {
            special = "nan";
        }
        else if (sci[read] == 'I')
        {
            special = negative ? "-inf" : "inf";
        }
        else
        {
            fprintf(stderr, "A number was written as \"%s\", which is neither digits nor Infinity nor NaN\n", sci);
            fixruntime_abort();
        }
        int length = fixruntime_check_float_text((int)strlen(special), size);
        memcpy(buf, special, (size_t)length + 1);
        return length;
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
    // Past the `E`: the power of ten the digits are multiplied by, of at most three digits and a
    // sign.
    read++;
    int exponent_negative = sci[read] == '-';
    if (exponent_negative)
    {
        read++;
    }
    int exponent = 0;
    for (; sci[read] != '\0'; read++)
    {
        exponent = exponent * 10 + (sci[read] - '0');
    }
    if (exponent_negative)
    {
        exponent = -exponent;
    }
    // Where the point falls among the digits: `10^(point-1) <= |v| < 10^point`.
    int point = exponent + 1;
    // The power of ten the digits, read as one whole number, are multiplied by.
    int scale = point - digit_count;

    // The text is built here first, so that its length is known before it is copied into `buf`.
    char text[FLOAT_TEXT_SIZE];
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
        // `point - 1` is the power of ten the first digit carries, which is what `sci` held.
        if (exponent < 0)
        {
            text[written++] = '-';
            exponent = -exponent;
        }
        written += (int)fixruntime_write_u64(text + written, (uint64_t)exponent);
    }
    text[written] = '\0';

    fixruntime_check_float_text(written, size);
    memcpy(buf, text, (size_t)written + 1);
    return written;
}

// Each of the two below writes the shortest text of `v` that reads back as `v` at `buf`, null-
// terminated, and reports how many bytes the text took, the null left out.
int64_t fixruntime_f32_to_str_shortest(char *buf, int64_t size, float v)
{
    _Static_assert(WIDEST_FLOAT_TEXT_SIZE(F32_POSITIONAL_LOW, F32_POSITIONAL_HIGH, F32_DIGITS) <=
                       FLOAT_TEXT_SIZE,
                   "an F32's window asks for more than the text buffer holds");
    char sci[32];
    sci[f2s_buffered_n(v, sci)] = '\0';
    return fixruntime_write_float_text(sci, buf, size, F32_POSITIONAL_LOW, F32_POSITIONAL_HIGH);
}

int64_t fixruntime_f64_to_str_shortest(char *buf, int64_t size, double v)
{
    _Static_assert(WIDEST_FLOAT_TEXT_SIZE(F64_POSITIONAL_LOW, F64_POSITIONAL_HIGH, F64_DIGITS) <=
                       FLOAT_TEXT_SIZE,
                   "an F64's window asks for more than the text buffer holds");
    char sci[32];
    sci[d2s_buffered_n(v, sci)] = '\0';
    return fixruntime_write_float_text(sci, buf, size, F64_POSITIONAL_LOW, F64_POSITIONAL_HIGH);
}

// The locale a number is read under: the one whose decimal point is the `.` Ryu writes.
//
// A program takes whatever locale its own code and the libraries it links set, and C's `strtod`
// reads the decimal point from that. Reading under this one instead is what makes a text `Std`
// wrote readable by the `Std` that wrote it.
static locale_t numeric_c_locale = (locale_t)0;

// Answers the locale numbers are read under, building it on the first call.
//
// Two threads reaching this together both build one, and the one that loses the exchange frees
// what it built, so the answer is the same object for every caller.
static locale_t float_text_locale(void)
{
    locale_t answer;
    __atomic_load(&numeric_c_locale, &answer, __ATOMIC_ACQUIRE);
    if (answer != (locale_t)0)
    {
        return answer;
    }
    answer = newlocale(LC_NUMERIC_MASK, "C", (locale_t)0);
    if (answer == (locale_t)0)
    {
        // POSIX gives every program the `C` locale, so there is no state in which this fails.
        fprintf(stderr, "The C locale, which numbers are read under, could not be built\n");
        fixruntime_abort();
    }
    locale_t none = (locale_t)0;
    if (!__atomic_compare_exchange_n(&numeric_c_locale, &none, answer, false, __ATOMIC_ACQ_REL,
                                     __ATOMIC_ACQUIRE))
    {
        freelocale(answer);
        answer = none;
    }
    return answer;
}

// Takes back the range error a number too small to hold in full raises.
//
// `strtod` raises `ERANGE` in two cases: the text names a number too large to hold, and it answers
// with an infinity; or it names one too small to hold in full, and it answers with the nearest
// number it can hold. The second is the number the text names — a subnormal number is a number
// like any other — so only the first is an error. A text naming a number too small to hold at all
// keeps the error, since zero is not what it names.
//
// # Arguments
// * `v` - What `strtod` answered.
static void fixruntime_keep_only_overflow(double v)
{
    if (errno == ERANGE && isfinite(v) && v != 0.0)
    {
        errno = 0;
    }
}

double fixruntime_strtod(const char *str)
{
    char *endptr;
    errno = 0;
    if (isspace((unsigned char)*str))
    {
        errno = EINVAL;
        return 0.0;
    }
    double v = strtod_l(str, &endptr, float_text_locale());
    fixruntime_keep_only_overflow(v);
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
    if (isspace((unsigned char)*str))
    {
        errno = EINVAL;
        return 0.0f;
    }
    float v = strtof_l(str, &endptr, float_text_locale());
    fixruntime_keep_only_overflow((double)v);
    if (endptr == str || *endptr != '\0')
    {
        errno = EINVAL;
    }
    return v;
}
