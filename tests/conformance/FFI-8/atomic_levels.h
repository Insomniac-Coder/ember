/* The C side of the FFI-8 reject test of `_Atomic T`: an atomic `float`, which `Atomic[T]` does
   not hold. */
#ifndef ATOMIC_LEVELS_H
#define ATOMIC_LEVELS_H
#include <stdatomic.h>
static inline float level_peek(_Atomic float* level) { return atomic_load(level); }
#endif
