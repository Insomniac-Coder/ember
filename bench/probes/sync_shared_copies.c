/* The C twin of bench/probes/sync_shared_copies.em: two counted blocks, a two-slot array of
 * pointers to them, and each copy into a slot an atomic increment of the new block's count and an
 * atomic decrement of the old one's (freeing at zero), with the same checked adds. */
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
typedef struct { uint32_t strong; int64_t value; } block;
static block* make(int64_t value) {
    block* b = malloc(sizeof *b);
    if (!b) abort();
    b->strong = 1;
    b->value = value;
    return b;
}
static void retain(block* b) { __atomic_fetch_add(&b->strong, 1u, __ATOMIC_RELAXED); }
static void release(block* b) {
    if (__atomic_fetch_sub(&b->strong, 1u, __ATOMIC_ACQ_REL) == 1u) free(b);
}
int main(void) {
    block* slots[2] = {make(1), make(2)};
    int64_t total = 0, n = 50000000;
    for (int64_t i = 0; i < n; ++i) {
        block* from = slots[(i + 1) % 2];
        retain(from);
        release(slots[i % 2]);
        slots[i % 2] = from;
        int64_t add = slots[0]->value + i % 3;
        if (__builtin_add_overflow(total, add, &total)) abort();
    }
    release(slots[0]);
    release(slots[1]);
    printf("%lld\n", (long long)total);
    return 0;
}
