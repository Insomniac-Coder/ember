#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

/* Ember's `%` is floor modulo (`[TYP-28]`). */
static int64_t floor_mod(int64_t a, int64_t m) {
    int64_t r = a % m;
    return r < 0 ? r + m : r;
}

static int64_t mix(int64_t* values, int64_t len, int64_t seed) {
    uint64_t acc = (uint64_t)seed;
    for (int64_t i = 0; i < len; i++) {
        uint64_t v = (uint64_t)values[i] * 31u + acc;
        values[i] = floor_mod((int64_t)v, 1000003);
        acc ^= v;
    }
    return (int64_t)acc;
}

int main(void) {
    int64_t len = 100000;
    int64_t* values = (int64_t*)malloc((size_t)len * sizeof *values);
    for (int64_t i = 0; i < len; i++) values[i] = i;
    uint64_t total = 0;
    for (int64_t round = 0; round < 4000; round++) total += (uint64_t)mix(values, len, round);
    printf("%lld\n", (long long)total);
    free(values);
    return 0;
}
