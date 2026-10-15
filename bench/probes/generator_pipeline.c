/* The C twin of bench/probes/generator_pipeline.em: the same filter, map and sum, and the same
 * stepped count down, as plain loops. */
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

int main(void) {
    int64_t n = 20000000;
    int64_t* xs = malloc((size_t)n * sizeof(int64_t));
    int64_t seed = 12345;
    for (int64_t i = 0; i < n; i++) {
        seed = (seed * 1103515245 + 12345) % 2147483648;
        xs[i] = seed % 1000;
    }
    int64_t total = 0;
    for (int64_t i = 0; i < n; i++) {
        int64_t x = xs[i];
        if (x % 7 != 0) total += x * 3;
    }
    int64_t stepped = 0;
    for (int64_t v = n * 3; v > 0; v -= 3) stepped += v % 11;
    printf("%lld %lld\n", (long long)total, (long long)stepped);
    free(xs);
    return 0;
}
