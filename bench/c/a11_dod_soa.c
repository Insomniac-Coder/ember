#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
int main(void) {
    int64_t n = 100000;
    double* xs = (double*)malloc(n * sizeof *xs);
    double* vs = (double*)malloc(n * sizeof *vs);
    for (int64_t i = 0; i < n; i++) { xs[i] = (double)i; vs[i] = 0.5; }
    for (int step = 0; step < 5000; step++)
        for (int64_t i = 0; i < n; i++) xs[i] = xs[i] + vs[i];
    double total = 0.0;
    for (int64_t i = 0; i < n; i++) total = total + xs[i];
    printf("%lld\n", (long long)total);
    return 0;
}
