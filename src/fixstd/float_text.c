/*
Reading a floating point number from text, for `Std::F64` and `Std::F32`.

A text is read by fast_float, carried under `ffc/`, which rounds the decimal number a text names to
the nearest number of the type, and takes `.` for the point whatever locale the program runs in.
*/

#include <inttypes.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

// fast_float's implementation, compiled into this translation unit alone. `ffc/ffc.h` declares
// every function it defines `static`, and the definitions take their linkage from those
// declarations, so none of its names reaches the link of a program, which may carry fast_float
// on its own.
#define FFC_IMPL
#include "ffc/ffc.h"

// Defined by the compiler, and declared in `runtime.c` as well; the two translation units carry
// the declaration because the runtime has no header of its own.
__attribute__((noreturn)) void fixruntime_abort(void);

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
