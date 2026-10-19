/* The C twin of bench/probes/simd_scale.em: the same arrays scaled by the same factors, as a C
 * programmer writes it, a plain loop the C compiler vectorises itself. */
#include <stdio.h>
#include <stdlib.h>

static void scale(const float* xs, float* out, size_t n, float k) {
    for (size_t i = 0; i < n; ++i) out[i] = xs[i] * k;
}

int main(void) {
    size_t n = 4096;
    float* xs = malloc(n * sizeof(float));
    float* out = malloc(n * sizeof(float));
    if (!xs || !out) abort();
    for (size_t i = 0; i < n; ++i) {
        xs[i] = (float)(i % 100);
        out[i] = 0.0f;
    }
    float k = 1.0f;
    for (long r = 0; r < 200000; ++r) {
        scale(xs, out, n, k);
        k = out[(r * 7) % (long)n] * 0.0001f + 1.0f;
    }
    double total = 0.0;
    for (size_t i = 0; i < n; ++i) total += (double)out[i];
    printf("%.1f\n", total);
    free(xs);
    free(out);
    return 0;
}
