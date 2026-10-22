/* The C side of the FFI-21 tests of callbacks with user data, declared by hand. */
#ifndef VISITS_H
#define VISITS_H
#include <stddef.h>

/* Calls `visit(data, value)` for each value: the user data first. */
void visit_each(const int* values, size_t count, void (*visit)(void* data, int value), void* data);
/* Sums `map(value, data)` over the values: the user data last, an integer. */
int sum_mapped(const int* values, size_t count, int (*map)(int value, ptrdiff_t data), ptrdiff_t data);
#endif
