/*
Writing a floating point number as text and reading one back, for `Std::F64` and `Std::F32`.

Ryu, whose sources sit beside this one under `ryu/`, finds the shortest digits that read back as a
finite number, from which the text is written here, and writes the text with a given number of
places behind the point. An infinity and a NaN are written here, as `inf`, `-inf` and `nan`. A text is read by
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

// The window the point is written positionally in: see `fixruntime_write_float_text`.
#define F32_POSITIONAL_LOW (-6)
#define F32_POSITIONAL_HIGH 13
#define F64_POSITIONAL_LOW (-5)
#define F64_POSITIONAL_HIGH 16

// Stops the program where a text of `length` bytes and its null do not fit the `size` bytes of its
// buffer.
//
// A buffer is sized from the widest text it can be asked to hold, so a text that does not fit
// means that size was derived wrongly. Stopping here names the two sizes, where letting the write
// run on would leave the heap damaged and the program going.
static inline __attribute__((always_inline)) void fixruntime_check_float_text_fits(int length, int64_t size)
{
    if ((int64_t)length + 1 > size)
    {
        fprintf(stderr, "A number's text takes %" PRId64 " bytes and its buffer holds %" PRId64 "\n",
                (int64_t)length + 1, size);
        fixruntime_abort();
    }
}

// Copies `text` to `buf`, null-terminated, and reports how many bytes the text took, the null
// left out. Stops the program where the text and its null do not fit `size`.
//
// # Arguments
// * `length` - The length of `text`, which needs no null of its own.
// * `size` - The bytes `buf` holds.
static int64_t fixruntime_copy_float_text(const char *text, int length, char *buf, int64_t size)
{
    fixruntime_check_float_text_fits(length, size);
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
// a power of ten, the same number takes fewer: a sign, a digit, a point, the places and `e308`.
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
// the number with a power of ten, `1.50e2`, and the others write it positionally, `150.00`. A
// precision of 0 writes `.0` after the digits, `2.0e2` and `200.0`. An `F32` is written as the
// `double` it widens to.
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

// Writes the `length` decimal digits of `mantissa` at `out`, the most significant first. No null
// follows them.
//
// # Examples
// `fixruntime_write_digits(out, 1234, 4)` writes `1234`.
static inline void fixruntime_write_digits(char *out, uint64_t mantissa, uint32_t length)
{
    uint32_t end = length;
    while (mantissa >= 100)
    {
        const uint32_t pair = (uint32_t)(mantissa % 100);
        mantissa /= 100;
        end -= 2;
        memcpy(out + end, DIGIT_TABLE + 2 * pair, 2);
    }
    if (mantissa >= 10)
    {
        memcpy(out, DIGIT_TABLE + 2 * mantissa, 2);
    }
    else
    {
        out[0] = (char)('0' + mantissa);
    }
}

// Writes the finite number other than zero that `negative` and `decimal` make up the way Fix spells
// it, at `buf`, null-terminated, and reports how many bytes the text took, the null left out.
//
// The digits are written positionally where the point falls inside or near them, and with a power
// of ten otherwise, so that the text stays about as wide as the digits it carries: `1.0e300` rather
// than a 1 followed by 300 zeros. Either way the text has a point and a digit on each side of it.
//
// # Arguments
// * `decimal` - The shortest digits that read back as the number, and the power of ten they are
//   multiplied by.
// * `size` - The bytes `buf` holds.
// * `positional_low`, `positional_high` - The window the point is written positionally in:
//   `positional_low < point <= positional_high`, where `point - 1` is the power of ten the first
//   digit carries. A wider window costs zeros, so it is drawn around the digits the type carries.
//   The widest text it allows is what sizes the buffer `to_string` passes in — must stay in sync
//   with the `size` the `ToString` implementations in `src/fixstd/std.fix` derive.
//
// # Examples
// With the window of an `F64`, `{ 1234, -2, 4 }` is written `12.34`, `{ 1234, 2, 4 }` is written
// `123400.0`, `{ 1234, -8, 4 }` is written `0.00001234`, and `{ 1, 300, 1 }` is written `1.0e300`.
static int64_t fixruntime_write_float_text(bool negative, ryu_decimal decimal, char *buf, int64_t size,
                                           int positional_low, int positional_high)
{
    const int count = (int)decimal.length;
    // Where the point falls among the digits: the first digit carries `10^(point-1)`.
    const int point = decimal.exponent + count;
    const bool positional = point > positional_low && point <= positional_high;
    // The power of ten written after the digits when the text is not positional.
    const int exponent = point - 1;
    const int exponent_magnitude = exponent < 0 ? -exponent : exponent;

    // The length is decided before anything is written, so that the text is written straight into
    // `buf` once it is known to fit.
    int length = negative;
    if (!positional)
    {
        // A digit, a point, the other digits or a `0`, `e`, the sign of the power and its digits.
        length += 2 + (count == 1 ? 1 : count - 1) + 1 + (exponent < 0) +
                  (exponent_magnitude >= 100 ? 3 : exponent_magnitude >= 10 ? 2 : 1);
    }
    else if (point >= count)
    {
        // The digits, the zeros up to the point, and `.0`.
        length += point + 2;
    }
    else if (point > 0)
    {
        // The digits with a point among them.
        length += count + 1;
    }
    else
    {
        // `0.`, the zeros after the point, and the digits.
        length += 2 - point + count;
    }
    fixruntime_check_float_text_fits(length, size);

    char *out = buf;
    if (negative)
    {
        *out++ = '-';
    }
    if (!positional)
    {
        // 1234e30 -> 1.234e33, and 1e30 -> 1.0e30. The digits are written one place to the right,
        // and the first of them is moved in front of the point.
        fixruntime_write_digits(out + 1, decimal.mantissa, (uint32_t)count);
        out[0] = out[1];
        out[1] = '.';
        int written = count + 1;
        if (count == 1)
        {
            out[written++] = '0';
        }
        out[written++] = 'e';
        if (exponent < 0)
        {
            out[written++] = '-';
        }
        fixruntime_write_exponent(out + written, exponent_magnitude);
    }
    else if (point >= count)
    {
        // 1234e3 -> 1234000.0
        fixruntime_write_digits(out, decimal.mantissa, (uint32_t)count);
        for (int i = count; i < point; i++)
        {
            out[i] = '0';
        }
        out[point] = '.';
        out[point + 1] = '0';
    }
    else if (point > 0)
    {
        // 1234e-2 -> 12.34. The digits are written one place to the right, and those before the
        // point are moved back in front of it.
        fixruntime_write_digits(out + 1, decimal.mantissa, (uint32_t)count);
        for (int i = 0; i < point; i++)
        {
            out[i] = out[i + 1];
        }
        out[point] = '.';
    }
    else
    {
        // 1234e-6 -> 0.001234
        out[0] = '0';
        out[1] = '.';
        for (int i = 0; i < -point; i++)
        {
            out[2 + i] = '0';
        }
        fixruntime_write_digits(out + 2 - point, decimal.mantissa, (uint32_t)count);
    }
    buf[length] = '\0';
    return length;
}

// Writes `v`, which is finite and zero, the way Fix spells it, at `buf`, null-terminated, and
// reports how many bytes the text took, the null left out.
static int64_t fixruntime_write_zero_text(bool negative, char *buf, int64_t size)
{
    return negative ? fixruntime_copy_float_text("-0.0", 4, buf, size) : fixruntime_copy_float_text("0.0", 3, buf, size);
}

// Each of the two below writes the shortest text of `v` that reads back as `v` at `buf`, null-
// terminated, and reports how many bytes the text took, the null left out.
int64_t fixruntime_f32_to_str_shortest(char *buf, int64_t size, float v)
{
    if (!isfinite(v))
    {
        return fixruntime_write_non_finite_text(v, buf, size);
    }
    if (v == 0.0f)
    {
        return fixruntime_write_zero_text(signbit(v), buf, size);
    }
    return fixruntime_write_float_text(signbit(v), f2s_shortest(v), buf, size, F32_POSITIONAL_LOW,
                                       F32_POSITIONAL_HIGH);
}

int64_t fixruntime_f64_to_str_shortest(char *buf, int64_t size, double v)
{
    if (!isfinite(v))
    {
        return fixruntime_write_non_finite_text(v, buf, size);
    }
    if (v == 0.0)
    {
        return fixruntime_write_zero_text(signbit(v), buf, size);
    }
    return fixruntime_write_float_text(signbit(v), d2s_shortest(v), buf, size, F64_POSITIONAL_LOW,
                                       F64_POSITIONAL_HIGH);
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
