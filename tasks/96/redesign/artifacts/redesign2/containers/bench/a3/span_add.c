/* A3 side question: what does losing Rust's `noalias` cost where it matters most, a vectorisable
   loop over two spans that might be the same memory? dst[k] = dst[k] * 0.5 + src[k] over short spans,
   50,000,000 calls of an out-of-line function. Spans never overlap here, so every mode does the same
   work and prints the same checksum.
     RESTRICT  : `restrict` pointers (what two Rust slices, one &mut, give LLVM).
     ALIAS     : plain pointers (two names may be one list: no information).
     CALLCHECK : the answer for spans: the caller compares the two ranges (two compares) and calls a
                 `restrict` copy of the function when they do not overlap, the plain copy when they do.
   KNOWN_LEN: the length is the constant 16 at every call (LLVM specialises the function for it);
   otherwise the length is read from a volatile, so it is unknown to the compiler. */
#include "../harness.h"
#define LEN 16
#if defined(KNOWN_LEN)
#define LENV LEN
#define LTAG "known"
#else
static volatile int len_source = LEN;
#define LENV n_dyn
#define LTAG "unknown"
#endif
static __attribute__((noinline)) void add_r(float *restrict dst, const float *restrict src, int n) {
    for (int k = 0; k < n; k++) dst[k] = dst[k] * 0.5f + src[k];
}
static __attribute__((noinline)) void add_a(float *dst, const float *src, int n) {
    for (int k = 0; k < n; k++) dst[k] = dst[k] * 0.5f + src[k];
}
#if defined(RESTRICT)
#define CALL(d, s, n) add_r(d, s, n)
#define MODE "restrict"
#elif defined(ALIAS)
#define CALL(d, s, n) add_a(d, s, n)
#define MODE "alias"
#else
/* overlap iff |d - s| < n: one unsigned compare, so one well-predicted branch. The first version,
   two short-circuit tests ((d + n <= s) || (s + n <= d)), mispredicted when the order of the two spans
   is random: measured 187 to 190 ms against 91 for RESTRICT (unknown length), single unpinned runs. */
#define CALL(d, s, n) (((uint64_t)((d) - (s) + (n) - 1) >= (uint64_t)(2 * (n) - 1)) ? add_r(d, s, n) : add_a(d, s, n))
#define MODE "callcheck"
#endif
int main(void) {
    h_start();
#if !defined(KNOWN_LEN)
    int n_dyn = len_source;
#endif
    static float pool[4096 + LEN];
    h_seed(3);
    for (int k = 0; k < 4096 + LEN; k++) pool[k] = (float)(h_rand() % 100) * 0.01f;
    for (int c = 0; c < 50000000; c++) {
        uint32_t a = (h_rand() % 128) * 32, b = (a + 16 + (h_rand() % 64) * 32) % 4096;  /* never overlap */
        CALL(&pool[a], &pool[b], LENV);
    }
    uint64_t s = 0;
    for (int k = 0; k < 4096; k++) s += (uint64_t)(int64_t)(pool[k] * 1000.0f);
    char name[64]; snprintf(name, sizeof name, "a3-span-%s-%s", MODE, LTAG);
    h_report(name, s);
    return 0;
}
