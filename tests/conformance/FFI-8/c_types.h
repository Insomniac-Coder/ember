/* The C side of the FFI-8 C-alias test: what this target's C compiler makes of its types. */
#ifndef C_TYPES_H
#define C_TYPES_H
#include <limits.h>
#include <stddef.h>
#include <wchar.h>
static inline size_t long_size(void) { return sizeof(long); }
static inline size_t wchar_size(void) { return sizeof(wchar_t); }
static inline char char_min(void) { return CHAR_MIN; }
static inline int char_min_int(void) { return CHAR_MIN; }
static inline wchar_t wchar_minus_one(void) { return (wchar_t)-1; }
static inline int wchar_is_signed(void) { return WCHAR_MIN < 0; }
static inline int add_ints(int a, int b) { return a + b; }
#endif
