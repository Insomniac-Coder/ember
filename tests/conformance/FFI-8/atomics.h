/* The C side of the FFI-8 tests of `_Atomic T`: a counter C bumps atomically, in a struct and
   through a pointer. */
#ifndef ATOMICS_H
#define ATOMICS_H
#include <stdatomic.h>
#include <stdint.h>
struct Counter { _Atomic uint32_t hits; uint32_t limit; };
static inline uint32_t counter_bump(struct Counter* counter) { return atomic_fetch_add(&counter->hits, 1) + 1; }
static inline uint32_t word_peek(_Atomic uint32_t* word) { return atomic_load(word); }
#endif
