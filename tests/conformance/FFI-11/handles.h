/* The C side of the FFI-11 tests of an opaque handle: a struct the header never defines. */
#ifndef HANDLES_H
#define HANDLES_H

typedef struct Counter Counter;

Counter* counter_new(int start);
void counter_free(Counter* counter);
int counter_get(const Counter* counter);
void counter_add(Counter* counter, int n);
const Counter* counter_same(const Counter* counter);
Counter* counter_same_mut(Counter* counter);
int counter_get_or(const Counter* counter, int fallback);
void counter_add_if(Counter* counter, int n);
int counter_freed(void);
#endif
