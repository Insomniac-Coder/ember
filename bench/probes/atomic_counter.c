/* The C twin of bench/probes/atomic_counter.em: a relaxed atomic add with the overflow check Ember
 * makes on its result. */
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
static int64_t hits;
int main(void) {
    int64_t n = 100000000;
    for (int64_t i = 0; i < n; ++i) {
        int64_t add = i % 3, sum;
        int64_t old = __atomic_fetch_add(&hits, add, __ATOMIC_RELAXED);
        if (__builtin_add_overflow(old, add, &sum)) abort();
    }
    printf("%lld\n", (long long)__atomic_load_n(&hits, __ATOMIC_SEQ_CST));
    return 0;
}
