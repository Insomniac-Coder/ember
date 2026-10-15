/* The C twin of bench/probes/stepped_range_value.em: the same count down, with Python's floor
 * `%` and a checked add, as Ember computes them (D-593). */
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
int main(void) {
    int64_t n = 60000000;
    int64_t stepped = 0;
    for (int64_t v = n * 3; v > 0; v -= 3) {
        int64_t r = v % 11; r += (r >> 63) & 11;
        if (__builtin_add_overflow(stepped, r, &stepped)) abort();
    }
    printf("%lld\n", (long long)stepped);
    return 0;
}
