#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

static void advance(double* xs, const double* vs, size_t n) {
    for (size_t i = 0; i < n; i++) xs[i] += vs[i];
}

int main(void) {
    size_t n = 100000;
    double* xs = malloc(n * sizeof(double));
    double* vs = malloc(n * sizeof(double));
    for (size_t i = 0; i < n; i++) {
        xs[i] = (double)i;
        vs[i] = 0.5;
    }
    for (int step = 0; step < 1000; step++) advance(xs, vs, n);
    double total = 0.0;
    for (size_t i = 0; i < n; i++) total += xs[i];
    printf("%lld\n", (long long)total);
    free(xs);
    free(vs);
    return 0;
}
