/* The C side of the FFI-29c tests. */
#ifndef WRAP_H
#define WRAP_H
#include <stdarg.h>

static inline int wrap_twice(int x) { return x * 2; }

/* Variadic and `static inline`: a wrapper cannot pass `...` on. */
static inline int wrap_sum(int count, ...) {
    va_list args;
    va_start(args, count);
    int total = 0;
    for (int i = 0; i < count; i++) total += va_arg(args, int);
    va_end(args);
    return total;
}
#endif
