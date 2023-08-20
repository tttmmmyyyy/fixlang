/*
C functions / values for implementing Fix standard library.
- When running program by `fix run`, then this source file will be compiled into shared library and loaded to the JIT environment.
- When running program by `fix build`, then this source file will be compiled into object file and linked to the binary.
*/

#include <stdio.h>
#include <stdint.h>
#include <inttypes.h>

// Print message to stderr, and flush it.
void fixruntime_eprint(const char *msg)
{
    fprintf(stderr, msg);
    fflush(stderr);
}

void fixruntime_u8_to_str(char *buf, uint8_t v)
{
    sprintf(buf, "%" PRIu8, v);
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

void fixruntime_f32_to_str(char *buf, float v)
{
    sprintf(buf, "%f", v);
}

void fixruntime_f64_to_str(char *buf, double v)
{
    sprintf(buf, "%lf", v);
}