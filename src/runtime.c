/*
C functions / values for implementing Fix standard library.
- When running program by `fix run`, then this source file will be compiled into shared library and loaded to the JIT environment.
- When running program by `fix build`, then this source file will be compiled into object file and linked to the binary.
*/

#include <stdio.h>

// Print message to stderr, and flush it.
void fixruntime_eprint(const char *msg)
{
    fprintf(stderr, msg);
    fflush(stderr);
}