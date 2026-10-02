/*
Writing a floating point number as text and reading one back, for `Std::F64` and `Std::F32`.

Every finite number's text is written by Ryu, whose sources sit beside this one under `ryu/`: the
shortest text that reads back as the number, and the text with a given number of places behind the
point. An infinity and a NaN are written here, as `inf`, `-inf` and `nan`. A text is read by
fast_float, carried under `ffc/`, which rounds the decimal number a text names to the nearest
number of the type, and takes `.` for the point whatever locale the program runs in.
*/

#include <inttypes.h>
#include <math.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdint.h>
#include <string.h>
#include "ryu/ryu.h"
#include "ryu/digit_table.h"

// fast_float's implementation, compiled into this translation unit alone. `ffc/ffc.h` declares
// every function it defines `static`, and the definitions take their linkage from those
// declarations, so none of its names reaches the link of a program, which may carry fast_float
// on its own.
#define FFC_IMPL
#include "ffc/ffc.h"

// Defined by the compiler, and declared in `runtime.c` as well; the two translation units carry
// the declaration because the runtime has no header of its own.
__attribute__((noreturn)) void fixruntime_abort(void);

// Writes the exponent of a number written with a power of ten, such as the `300` of `1.0e300`, at
// `buf` in decimal, and reports how many digits it took. No null follows them.
//
// `exponent` is from 0 to 999. The caller writes the `-` of a negative exponent before calling.
//
// # Examples
// `fixruntime_write_exponent(buf, 300)` writes `300` and returns 3, and
// `fixruntime_write_exponent(buf, 7)` writes `7` and returns 1.
static int fixruntime_write_exponent(char *buf, int exponent)
{
    if (exponent >= 100)
    {
        memcpy(buf, DIGIT_TABLE + 2 * (exponent / 10), 2);
        buf[2] = (char)('0' + exponent % 10);
        return 3;
    }
    if (exponent >= 10)
    {
        memcpy(buf, DIGIT_TABLE + 2 * exponent, 2);
        return 2;
    }
    buf[0] = (char)('0' + exponent);
    return 1;
}

// The bytes `fixruntime_write_float_text` builds a text in. The static assertions at the two
// entry points hold each window to what fits here, since a wider one would write past it.
#define FLOAT_TEXT_SIZE 48

// The bytes a window's text takes at its widest, the null included. The sum counts a sign, two
// bytes, the zeros either edge of the window allows beside the digits, the digits themselves, four
// bytes, and a null. A text written positionally spends the two bytes on a point and the `0` beside
// it, and leaves the four unused. A text written with a power of ten spends the two bytes on a point
// and the `e`, and the four on the rest of the power, `-324` at its widest; it leaves the zeros
// unused. A single digit is followed by a point and a `0`, and that `0` fits in the bytes counted
// for the digits.
#define WIDEST_FLOAT_TEXT_SIZE(low, high, digits) \
    (1 + 2 + ((-(low)) > (high) ? (-(low)) : (high)) + (digits) + 4 + 1)

// The window the point is written positionally in, and the digits the type takes at its widest.
#define F32_POSITIONAL_LOW (-6)
#define F32_POSITIONAL_HIGH 13
#define F32_DIGITS 9
#define F64_POSITIONAL_LOW (-5)
#define F64_POSITIONAL_HIGH 16
#define F64_DIGITS 17

// Copies `text` to `buf`, null-terminated, and reports how many bytes the text took, the null
// left out. Stops the program where the text and its null do not fit `size`.
//
// A buffer is sized from the widest text it can be asked to hold, so a text that does not fit
// means that size was derived wrongly. Stopping here names the two sizes, where letting the write
// run on would leave the heap damaged and the program going.
//
// # Arguments
// * `length` - The length of `text`, which needs no null of its own.
// * `size` - The bytes `buf` holds.
//
// Every text written passes through here once, so it is inlined into each writer: a call costs
// `Std::F64::to_string` 8 more instructions.
static inline __attribute__((always_inline)) int64_t fixruntime_copy_float_text(const char *text, int length,
                                                                                 char *buf, int64_t size)
{
    if ((int64_t)length + 1 > size)
    {
        fprintf(stderr, "A number's text takes %" PRId64 " bytes and its buffer holds %" PRId64 "\n",
                (int64_t)length + 1, size);
        fixruntime_abort();
    }
    memcpy(buf, text, (size_t)length);
    buf[length] = '\0';
    return length;
}

// Writes `v`, which is an infinity or a NaN, at `buf` the way Fix spells it, null-terminated, and
// reports how many bytes the text took, the null left out.
//
// Fix writes `inf`, `-inf` and `nan`, which is what `Std::FromString` takes back. A NaN is `nan`
// whatever its sign.
static int64_t fixruntime_write_non_finite_text(double v, char *buf, int64_t size)
{
    const char *text = isnan(v) ? "nan" : v < 0 ? "-inf" : "inf";
    return fixruntime_copy_float_text(text, (int)strlen(text), buf, size);
}

// The bytes a text with a given number of places behind the point takes at its widest, the null
// left out: the most negative `F64` written positionally with the 255 places a `U8` precision
// reaches, which is a sign, the 309 digits of its whole part, a point and the places. Written with
// a power of ten, the same number takes fewer: a sign, a digit, a point, the places and `e+308`.
// Ryu writes with no bound, so this size is the whole guard; `test_float_to_string_precision`
// writes that widest text and pins its length.
#define PRECISION_TEXT_SIZE (1 + 309 + 1 + 255)

// Writes `v` at `buf` with `precision` digits after the point, null-terminated, and reports how
// many bytes the text took, the null left out.
//
// # Arguments
// * `write_finite` - The Ryu printf function that writes a finite `v`: `d2fixed_buffered_n` or
//   `d2exp_buffered_n`.
static int64_t fixruntime_write_precision_text(int (*write_finite)(double, uint32_t, char *), double v,
                                               uint8_t precision, char *buf, int64_t size)
{
    if (!isfinite(v))
    {
        return fixruntime_write_non_finite_text(v, buf, size);
    }
    char text[PRECISION_TEXT_SIZE];
    return fixruntime_copy_float_text(text, write_finite(v, precision, text), buf, size);
}

// Each of the four below writes `v` at `buf` with `precision` digits after the point, null-
// terminated, and reports how many bytes the text took, the null left out. The `exp` ones write
// the number in scientific notation, and the others write it positionally. A finite number's text
// is the one C's `printf` writes for `%.*e` and `%.*f` under the `C` locale; an `F32` is written as
// the `double` it widens to, as `printf` takes it.
int64_t fixruntime_f32_to_str_exp_precision(char *buf, int64_t size, float v, uint8_t precision)
{
    return fixruntime_write_precision_text(d2exp_buffered_n, (double)v, precision, buf, size);
}

int64_t fixruntime_f32_to_str_precision(char *buf, int64_t size, float v, uint8_t precision)
{
    return fixruntime_write_precision_text(d2fixed_buffered_n, (double)v, precision, buf, size);
}

int64_t fixruntime_f64_to_str_exp_precision(char *buf, int64_t size, double v, uint8_t precision)
{
    return fixruntime_write_precision_text(d2exp_buffered_n, v, precision, buf, size);
}

int64_t fixruntime_f64_to_str_precision(char *buf, int64_t size, double v, uint8_t precision)
{
    return fixruntime_write_precision_text(d2fixed_buffered_n, v, precision, buf, size);
}

// Writes the scientific text Ryu produced the way Fix spells a floating point number, and
// reports how many bytes the text took.
//
// `sci` holds what `d2s_buffered_n` or `f2s_buffered_n` wrote for a finite number: a sign, the
// shortest digits that read back as the number with a point after the first of them, `E`, and the
// power of ten those digits are multiplied by. Fix writes those digits positionally where the
// point falls inside or near them, and as a power of ten otherwise, so that the text stays about
// as wide as the digits it carries: `1.0e300` rather than a 1 followed by 300 zeros. Either way the
// text has a point, as a floating point literal of Fix does.
//
// # Arguments
// * `sci` - The scientific text, null-terminated.
// * `buf` - Where the text is written, null-terminated.
// * `size` - The bytes `buf` holds.
// * `positional_low`, `positional_high` - The window the point is written positionally in:
//   `positional_low < point <= positional_high`, where `point - 1` is the power of ten the first
//   of the shortest digits carries. A wider window costs zeros, so it is drawn around the digits
//   the type carries, and the widest text it allows is what sizes the buffer `to_string` passes
//   in — must stay in sync with the `size` the `ToString` implementations in `src/fixstd/std.fix`
//   derive.
static int64_t fixruntime_write_float_text(const char *sci, char *buf, int64_t size,
                                                int positional_low, int positional_high)
{
    int read = 0;
    int negative = sci[0] == '-';
    if (negative)
    {
        read = 1;
    }

    if (sci[read] < '0' || sci[read] > '9')
    {
        fprintf(stderr, "Ryu wrote a finite number as \"%s\", which does not open with a digit\n", sci);
        fixruntime_abort();
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
    // Where the point falls among the digits: the first digit carries `10^(point-1)`.
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
        // 1234e30 -> 1.234e33, and 1e30 -> 1.0e30
        text[written++] = digits[0];
        text[written++] = '.';
        if (digit_count > 1)
        {
            memcpy(text + written, digits + 1, (size_t)(digit_count - 1));
            written += digit_count - 1;
        }
        else
        {
            text[written++] = '0';
        }
        text[written++] = 'e';
        // `point - 1` is the power of ten the first digit carries, which is what `sci` held.
        if (exponent < 0)
        {
            text[written++] = '-';
            exponent = -exponent;
        }
        written += fixruntime_write_exponent(text + written, exponent);
    }

    return fixruntime_copy_float_text(text, written, buf, size);
}

// Each of the two below writes the shortest text of `v` that reads back as `v` at `buf`, null-
// terminated, and reports how many bytes the text took, the null left out.
int64_t fixruntime_f32_to_str_shortest(char *buf, int64_t size, float v)
{
    _Static_assert(WIDEST_FLOAT_TEXT_SIZE(F32_POSITIONAL_LOW, F32_POSITIONAL_HIGH, F32_DIGITS) <=
                       FLOAT_TEXT_SIZE,
                   "an F32's window asks for more than the text buffer holds");
    if (!isfinite(v))
    {
        return fixruntime_write_non_finite_text(v, buf, size);
    }
    char sci[32];
    sci[f2s_buffered_n(v, sci)] = '\0';
    return fixruntime_write_float_text(sci, buf, size, F32_POSITIONAL_LOW, F32_POSITIONAL_HIGH);
}

int64_t fixruntime_f64_to_str_shortest(char *buf, int64_t size, double v)
{
    _Static_assert(WIDEST_FLOAT_TEXT_SIZE(F64_POSITIONAL_LOW, F64_POSITIONAL_HIGH, F64_DIGITS) <=
                       FLOAT_TEXT_SIZE,
                   "an F64's window asks for more than the text buffer holds");
    if (!isfinite(v))
    {
        return fixruntime_write_non_finite_text(v, buf, size);
    }
    char sci[32];
    sci[d2s_buffered_n(v, sci)] = '\0';
    return fixruntime_write_float_text(sci, buf, size, F64_POSITIONAL_LOW, F64_POSITIONAL_HIGH);
}

// How the last reading of a number on this thread came out: one of the three `FLOAT_TEXT_*`
// values below. `fixruntime_read_f64` and `fixruntime_read_f32` answer with the number alone, and
// `std.fix` reads this through `fixruntime_float_text_outcome` right after each call.
//
// The values must stay in sync with `Std::String::_float_text_result` in `std.fix`, which tells
// them apart.
static _Thread_local uint8_t float_text_outcome;

// The text named a number, and the answer is it, rounded to the nearest number of the type.
#define FLOAT_TEXT_READ 0
// The text is not a number, or holds something after one.
#define FLOAT_TEXT_MALFORMED 1
// The text names a finite number whose nearest number of the type is an infinity, or a number other
// than zero whose nearest number of the type is zero.
#define FLOAT_TEXT_OUT_OF_RANGE 2

// Answers with how the last reading of a number on this thread came out.
uint8_t fixruntime_float_text_outcome(void)
{
    return float_text_outcome;
}

// Records in `float_text_outcome` how a reading by fast_float came out.
//
// The text names a number and nothing else, so a reading that stopped before `end` is malformed.
// fast_float reports a number too large or too small to hold as out of range, and still answers
// with the infinity or the zero it rounds to.
//
// # Arguments
// * `result` - What fast_float answered.
// * `end` - The end of the text.
static void fixruntime_record_float_text_outcome(ffc_result result, const char *end)
{
    switch (result.outcome)
    {
    case FFC_OUTCOME_OK:
        float_text_outcome = result.ptr == end ? FLOAT_TEXT_READ : FLOAT_TEXT_MALFORMED;
        break;
    case FFC_OUTCOME_OUT_OF_RANGE:
        float_text_outcome = result.ptr == end ? FLOAT_TEXT_OUT_OF_RANGE : FLOAT_TEXT_MALFORMED;
        break;
    case FFC_OUTCOME_INVALID_INPUT:
        float_text_outcome = FLOAT_TEXT_MALFORMED;
        break;
    default:
        fprintf(stderr, "fast_float answered with an outcome it does not define: %" PRIu32 "\n",
                (uint32_t)result.outcome);
        fixruntime_abort();
    }
}

// The texts `fixruntime_read_f64` and `fixruntime_read_f32` read: a decimal number with an optional
// sign, point and power of ten, `inf`, `infinity` and `nan` in any case, with an optional sign, and
// `nan` followed by a parenthesized run of letters, digits and underscores. A NaN read is the
// quiet NaN of the sign written.
static ffc_parse_options fixruntime_float_text_options(void)
{
    ffc_parse_options options = {
        .format = FFC_PRESET_GENERAL | FFC_FORMAT_FLAG_ALLOW_LEADING_PLUS,
        .decimal_point = '.',
    };
    return options;
}

// Reads a `double` from the whole of the null-terminated `str`, and records how the reading came
// out in `float_text_outcome`. Answers with 0 where that outcome is other than
// `FLOAT_TEXT_READ`.
double fixruntime_read_f64(const char *str)
{
    const char *end = str + strlen(str);
    double v;
    ffc_result result =
        ffc_from_chars_double_options(str, end, &v, fixruntime_float_text_options());
    fixruntime_record_float_text_outcome(result, end);
    return float_text_outcome == FLOAT_TEXT_READ ? v : 0.0;
}

// Reads a `float` from the whole of the null-terminated `str`, and records how the reading came out
// in `float_text_outcome`. Answers with 0 where that outcome is other than `FLOAT_TEXT_READ`.
float fixruntime_read_f32(const char *str)
{
    const char *end = str + strlen(str);
    float v;
    ffc_result result = ffc_from_chars_float_options(str, end, &v, fixruntime_float_text_options());
    fixruntime_record_float_text_outcome(result, end);
    return float_text_outcome == FLOAT_TEXT_READ ? v : 0.0f;
}
