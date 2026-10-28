/* plain_samevar.c: workload samevar, plain C. A function that changes two float spans,
       dst[k] = dst[k] * 0.5 + src[k] * 0.25
       src[k] = src[k] * 0.75 + 0.125
   called 50,000,128 times on 16-float spans (the length read from a volatile, so not known when
   compiling) (the shape of run 2's span_add, AN-14's 1.53x case:
   artifacts/redesign2/containers/bench/a3/span_add.c), out of line.

   Argument `one`: every call gives the function one span twice (dst and src are the same memory).
   Argument `two`: every call gives it a span of `left` and a span of `right` (never the same memory).
   Calls: 195,313 rounds; in round r, for j = 0 .. 255, dst = left[16 j ..], src = right[16 p ..] with
   p = (97 j + r) mod 256 (`two`), or dst = src = left[16 j ..] (`one`).

   What a careful C programmer writes: one function with plain pointers, since some calls pass one span
   twice; `restrict` would make those calls undefined. -DNOALIAS_ARGS builds the `restrict` version,
   valid for `two` only (the program refuses `one`): what Rust's two `&mut` give LLVM. */
#include "m4.h"

#define SPAN 16
#define POOL 4096
#define ROUNDS 195313

#ifdef NOALIAS_ARGS
#define NA RESTRICT
#define NAME "plain_samevar_noalias"
#else
#define NA
#define NAME "plain_samevar"
#endif

static NOINLINE void blend(float *NA dst, float *NA src, int64_t n) {
    for (int64_t k = 0; k < n; k++) {
        dst[k] = dst[k] * 0.5f + src[k] * 0.25f;
        src[k] = src[k] * 0.75f + 0.125f;
    }
}

static float left[POOL], right[POOL];
static volatile int64_t span_len = SPAN;   /* read once: the length is not known when compiling (run 2's "unknown") */

int main(int argc, char **argv) {
    h_start();
    int one = argc > 1 && strcmp(argv[1], "one") == 0;
#ifdef NOALIAS_ARGS
    if (one) { fprintf(stderr, "plain_samevar_noalias: `one` would be undefined behaviour\n"); return 2; }
#endif
    int64_t n = span_len;
    h_seed(31);
    for (int k = 0; k < POOL; k++) { left[k] = (float)(h_rand() % 1000) * 0.001f; right[k] = (float)(h_rand() % 1000) * 0.001f; }
    for (int r = 0; r < ROUNDS; r++) {
        for (int j = 0; j < 256; j++) {
            if (one) blend(&left[j * SPAN], &left[j * SPAN], n);
            else blend(&left[j * SPAN], &right[((j * 97 + r) & 255) * SPAN], n);
        }
    }
    uint64_t s = 0;
    for (int k = 0; k < POOL; k++) s += (uint64_t)(int64_t)(left[k] * 1000000.0f) + (uint64_t)(int64_t)(right[k] * 1000000.0f) * 3u;
    char name[64]; snprintf(name, sizeof name, "%s-%s", NAME, one ? "one" : "two");
    h_report(name, s);
    return 0;
}
