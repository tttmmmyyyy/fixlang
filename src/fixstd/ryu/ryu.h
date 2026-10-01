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
#ifndef RYU_H
#define RYU_H

#ifdef __cplusplus
extern "C" {
#endif

#include <inttypes.h>

// Modified from upstream Ryu by the Fix project: this file declares the four functions the Fix
// runtime calls. `ryu_decimal`, `d2s_shortest` and `f2s_shortest` are the Fix project's.
//
// The runtime links Ryu into every program it builds, and a program may carry Ryu on its own, so
// each function here takes a name that begins with `fixruntime_`, which no name of such a program
// does.
#define d2s_shortest fixruntime_ryu_d2s_shortest
#define f2s_shortest fixruntime_ryu_f2s_shortest
#define d2fixed_buffered_n fixruntime_ryu_d2fixed_buffered_n
#define d2exp_buffered_n fixruntime_ryu_d2exp_buffered_n

// A positive number written in decimal: `mantissa * 10^exponent`, where `mantissa` has `length`
// digits.
typedef struct ryu_decimal {
  uint64_t mantissa;
  int32_t exponent;
  uint32_t length;
} ryu_decimal;

// Each of the two below gives the shortest decimal that reads back as `f`, which is finite and
// other than zero. The sign of `f` is left out.
//
// For example, `d2s_shortest(-0.25)` is `{ 25, -2, 2 }`, and `d2s_shortest(1e300)` is
// `{ 1, 300, 1 }`.
ryu_decimal d2s_shortest(double f);
ryu_decimal f2s_shortest(float f);

// Each of the two below writes the finite `d` at `result` with `precision` digits after the point,
// and reports how many bytes the text took. No null follows it.
//
// `d2fixed_buffered_n` writes the number positionally, and `d2exp_buffered_n` writes one digit
// before the point and a power of ten after the digits. The power of ten is written as Fix writes
// it, with `-` for a negative one and no padding: `1.50e2`, `1.50e-2`. With a precision of 0, the
// point is written all the same, followed by one `0`: `2.0`, `2.0e2`. So every text these write is
// a floating point literal of Fix.
int d2fixed_buffered_n(double d, uint32_t precision, char* result);
int d2exp_buffered_n(double d, uint32_t precision, char* result);

#ifdef __cplusplus
}
#endif

#endif // RYU_H
