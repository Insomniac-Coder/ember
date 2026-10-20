/* The C side of the FFI-2a tests: `const` says which pointers C only reads. */
#ifndef BUFFERS_H
#define BUFFERS_H
#include <stddef.h>
static inline int buffers_first(const int* items, size_t n) { return n > 0 ? items[0] : -1; }
static inline void buffers_zero(int* items, size_t n) { for (size_t i = 0; i < n; i++) items[i] = 0; }
static inline const int* buffers_pick(const int* items, size_t n) { return n > 1 ? &items[1] : items; }
#endif
