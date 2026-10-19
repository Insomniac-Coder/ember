/* The C side of the FFI-12 tests. */
#ifndef TALLY_H
#define TALLY_H
#include <stddef.h>
static inline int tally_sum(const int* items, size_t count) {
    int total = 0;
    for (size_t i = 0; i < count; i++) total += items[i];
    return total;
}
static inline int tally_scale(int value, int by) { return value * by; }
typedef struct Tally Tally;
#endif
