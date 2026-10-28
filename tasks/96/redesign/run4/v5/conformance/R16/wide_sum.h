/* The C side of W6's R16 tests of a variadic call (E5013): a C function that reads its further
   arguments as `int64_t`, as `tests/conformance/FFI-8/variadic.c`'s `sum_wide` does. */
#ifndef WIDE_SUM_H
#define WIDE_SUM_H
#include <stdint.h>
/* The sum of `count` further `int64_t` arguments. */
int64_t sum_wide(int count, ...);
#endif
