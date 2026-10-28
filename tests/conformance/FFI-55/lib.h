/* The C side of the FFI-55 tests: the header another package's overlays are applied to. */
#ifndef LIB_H
#define LIB_H
static inline int lib_twice(int x) { return 2 * x; }
#endif
