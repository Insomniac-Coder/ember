#pragma once
/* The header of `[DIA-5]`'s cases: each error about one of these declarations names it as it is
 * written here, and where. */
#include <stddef.h>

int label_length(const char *text);

int label_sum(const int *items,
              size_t n);

static inline int label_twice(int value) {
    return value * 2;
}
