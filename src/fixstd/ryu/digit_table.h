// Copyright 2018 Ulf Adams
//
// The contents of this file may be used under the terms of the Apache License,
// Version 2.0.
//
//    (See accompanying file LICENSE-Apache or copy at
//     http://www.apache.org/licenses/LICENSE-2.0)
//
// Alternatively, the contents of this file may be used under the terms of
// the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE-Boost or copy at
//     https://www.boost.org/LICENSE_1_0.txt)
//
// Unless required by applicable law or agreed to in writing, this software
// is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.
#ifndef RYU_DIGIT_TABLE_H
#define RYU_DIGIT_TABLE_H

#include <stdint.h>
#include <string.h>

// A table of all two-digit numbers. This is used to speed up decimal digit
// generation by copying pairs of digits into the final output.
static const char DIGIT_TABLE[200] = {
  '0','0','0','1','0','2','0','3','0','4','0','5','0','6','0','7','0','8','0','9',
  '1','0','1','1','1','2','1','3','1','4','1','5','1','6','1','7','1','8','1','9',
  '2','0','2','1','2','2','2','3','2','4','2','5','2','6','2','7','2','8','2','9',
  '3','0','3','1','3','2','3','3','3','4','3','5','3','6','3','7','3','8','3','9',
  '4','0','4','1','4','2','4','3','4','4','4','5','4','6','4','7','4','8','4','9',
  '5','0','5','1','5','2','5','3','5','4','5','5','5','6','5','7','5','8','5','9',
  '6','0','6','1','6','2','6','3','6','4','6','5','6','6','6','7','6','8','6','9',
  '7','0','7','1','7','2','7','3','7','4','7','5','7','6','7','7','7','8','7','9',
  '8','0','8','1','8','2','8','3','8','4','8','5','8','6','8','7','8','8','8','9',
  '9','0','9','1','9','2','9','3','9','4','9','5','9','6','9','7','9','8','9','9'
};

// Modified from upstream Ryu by the Fix project: the two functions below write the power of ten of a
// number written with one, as Fix writes it: `-` for a negative power, no `+`, and no padding.

// The bytes `append_exponent` writes for `exp`, which is from -999 to 999.
//
// For example, `exponent_length(-324)` is 4, and `exponent_length(7)` is 1.
static inline int exponent_length(const int32_t exp) {
  const int32_t magnitude = exp < 0 ? -exp : exp;
  return (exp < 0) + (magnitude >= 100 ? 3 : magnitude >= 10 ? 2 : 1);
}

// Writes `exp`, which is from -999 to 999, at `result`, and reports how many bytes it took. No null
// follows them.
//
// For example, `append_exponent(-324, result)` writes `-324`, and `append_exponent(7, result)`
// writes `7`.
static inline int append_exponent(int32_t exp, char* const result) {
  int index = 0;
  if (exp < 0) {
    result[index++] = '-';
    exp = -exp;
  }
  if (exp >= 100) {
    memcpy(result + index, DIGIT_TABLE + 2 * (exp / 10), 2);
    result[index + 2] = (char) ('0' + exp % 10);
    return index + 3;
  }
  if (exp >= 10) {
    memcpy(result + index, DIGIT_TABLE + 2 * exp, 2);
    return index + 2;
  }
  result[index] = (char) ('0' + exp);
  return index + 1;
}

#endif // RYU_DIGIT_TABLE_H
