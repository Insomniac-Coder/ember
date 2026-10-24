/* A3 span kernel (workload: ../../containers/bench/a3/span_add.c, length unknown to the compiler),
   the answer AS EMBER WOULD EMIT IT, which the report's CALLCHECK model leaves part of out:
   - pool[a..a+n] and pool[b..b+n] are slices of a list, so each is bounds-checked ([PAN-1]); the
     report's models take &pool[a] unchecked, while its safe Rust twins pay these checks;
   - saved form of the overlap rule: when the two ranges overlap, the read-only one is copied to a
     temporary first and the one `restrict` copy of the function is called (numpy's meaning, "as if
     the inputs were copied first"; Python list slices are copies). So the function is compiled once,
     not twice. The ranges never overlap in this workload, so the copy never runs. */
#include "../harness.h"
#include <string.h>
#define LEN 16
static volatile int len_source = LEN;
static volatile size_t pool_len_source = 4096 + LEN;
static void panic(const char *m) { fprintf(stderr, "panic: %s\n", m); exit(101); }
static __attribute__((noinline)) void add_r(float *restrict dst, const float *restrict src, int n) {
    for (int k = 0; k < n; k++) dst[k] = dst[k] * 0.5f + src[k];
}
static inline void call_add(float *pool, size_t pool_len, size_t a, size_t b, int n) {
    if (__builtin_expect(a + (size_t)n > pool_len || b + (size_t)n > pool_len, 0)) panic("slice out of bounds");
    float *d = pool + a; const float *s = pool + b;
    if (__builtin_expect((uint64_t)(d - s + n - 1) < (uint64_t)(2 * n - 1), 0)) {   /* overlap */
        float tmp[64];
        if (n > 64) panic("temporary too small in this model");
        memcpy(tmp, s, (size_t)n * sizeof(float));
        s = tmp;
    }
    add_r(d, s, n);
}
int main(void) {
    h_start();
    int n_dyn = len_source;
    size_t pool_len = pool_len_source;
    static float pool[4096 + LEN];
    h_seed(3);
    for (int k = 0; k < 4096 + LEN; k++) pool[k] = (float)(h_rand() % 100) * 0.01f;
    for (int c = 0; c < 50000000; c++) {
        uint32_t a = (h_rand() % 128) * 32, b = (a + 16 + (h_rand() % 64) * 32) % 4096;
        call_add(pool, pool_len, a, b, n_dyn);
    }
    uint64_t s = 0;
    for (int k = 0; k < 4096; k++) s += (uint64_t)(int64_t)(pool[k] * 1000.0f);
    h_report("a3-span-answer-checked-unknown", s);
    return 0;
}
