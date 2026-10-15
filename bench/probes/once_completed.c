/* The C twin of bench/probes/once_completed.em: a run-once flag checked with an acquire load on
 * every call, as Ember's `Once.call_once` does, with the same checked adds; the flag and the total
 * are locals, as Ember's are. */
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
static inline void call_once(uint32_t* state, int64_t* total) {
    if (__atomic_load_n(state, __ATOMIC_ACQUIRE) == 2) return;
    uint32_t expected = 0;
    if (__atomic_compare_exchange_n(state, &expected, 1, 0, __ATOMIC_ACQUIRE, __ATOMIC_ACQUIRE)) {
        if (__builtin_add_overflow(*total, 1, total)) abort();
        __atomic_store_n(state, 2, __ATOMIC_RELEASE);
    }
}
int main(void) {
    int64_t n = 100000000;
    int64_t total = 0;
    uint32_t state = 0;
    for (int64_t i = 0; i < n; ++i) {
        call_once(&state, &total);
        if (__builtin_add_overflow(total, i % 3, &total)) abort();
    }
    printf("%lld\n", (long long)total);
    return 0;
}
