#include "visits.h"

void visit_each(const int* values, size_t count, void (*visit)(void* data, int value), void* data) {
    for (size_t i = 0; i < count; i++) visit(data, values[i]);
}

int sum_mapped(const int* values, size_t count, int (*map)(int value, ptrdiff_t data), ptrdiff_t data) {
    int total = 0;
    for (size_t i = 0; i < count; i++) total += map(values[i], data);
    return total;
}
