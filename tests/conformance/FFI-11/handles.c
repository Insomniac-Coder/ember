/* The C side of the FFI-11 tests of an opaque handle. */
#include <stdlib.h>
#include "handles.h"

struct Counter {
    int value;
};

static int freed = 0;

Counter* counter_new(int start) {
    Counter* counter = malloc(sizeof *counter);
    counter->value = start;
    return counter;
}
void counter_free(Counter* counter) {
    free(counter);
    freed += 1;
}
int counter_get(const Counter* counter) { return counter->value; }
void counter_add(Counter* counter, int n) { counter->value += n; }
const Counter* counter_same(const Counter* counter) { return counter; }
Counter* counter_same_mut(Counter* counter) { return counter; }
int counter_get_or(const Counter* counter, int fallback) { return counter ? counter->value : fallback; }
void counter_add_if(Counter* counter, int n) {
    if (counter) counter->value += n;
}
int counter_freed(void) { return freed; }
