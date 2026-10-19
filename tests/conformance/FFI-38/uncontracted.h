/* The C side of the FFI-38 tests: nothing here states who owns a pointer or how far it reaches. */
#ifndef UNCONTRACTED_H
#define UNCONTRACTED_H
static inline int uncontracted_add(int a, int b) { return a + b; }
static inline int uncontracted_sum(const int* items, int count) {
    int total = 0;
    for (int i = 0; i < count; i++) total += items[i];
    return total;
}
#endif
