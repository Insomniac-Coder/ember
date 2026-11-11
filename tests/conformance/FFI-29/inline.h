/* The C side of the FFI-29 tests. */
#ifndef INLINE_H
#define INLINE_H
#include <stdarg.h>

/* `static inline`: no symbol to link, so the shim wraps it. */
static inline int inline_twice(int x) { return x * 2; }

/* An ordinary extern function; its definition is in the header's source. */
int inline_extern(int x);
#endif
