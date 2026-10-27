/* model_samevar.c: workload samevar, what Ember emits under the rule as written (verdict 88;
   R4 exception 1; [BRW-11] exception 1, [BRW-22], [SIMD-3] (e)).

       fn blend(ref mut dst: MutSpan[f32], ref mut src: MutSpan[f32]):
           for k in 0..dst.len():
               dst[k] = dst[k] * 0.5 + src[k] * 0.25
               src[k] = src[k] * 0.75 + 0.125

       # in main, n = 16 read at run time, a = 16 j, b = 16 ((97 j + r) mod 256):
       if one:
           s = ref mut left[a .. a + n].as_mut_span()
           blend(ref mut s, ref mut s)                  # one whole variable under two names
       else:
           blend(ref mut left[a .. a + n].as_mut_span(), ref mut right[b .. b + n].as_mut_span())

   The `one` call names the whole variable `s` twice, which the compiler proves when compiling, so it
   is allowed and compiled against a copy of `blend` in which the two names are one (`blend__one`);
   the `two` call uses the normal copy, where each `ref mut` parameter is the only way to its place
   and is compiled with noalias. The two lists are different owners, so `two` needs no
   `assert_disjoint` ([SIMD-3] (b)). Inside one package the compiler sees that `blend` never points
   `dst` or `src` at another span, so each span is passed as its two words (run4/v5/area-A.md near
   line 2507).

   Checks kept: [TYP-31] making each span checks `a + n <= left.len()` (the slice's range); [OPT-2] the
   normal copy's loop indexes `dst[k]` (in range by the loop) and `src[k]`: one entry test
   `src.len() >= dst.len()`, the unchecked body when it passes, the checked body otherwise. In the
   one-name copy `src` is `dst`, so that test is `n >= n` and goes. No integer arithmetic to check.
   -DGENERIC_COPY (salvage_samevar_generic.c) sends the `one` call to a copy with no noalias instead. */
#include "m4.h"

#define SPAN 16
#define POOL 4096
#define ROUNDS 195313

/* The normal copy: two `ref mut` spans, each noalias ([BRW-22]), each as its two words. */
static NOINLINE void blend(float *RESTRICT dst, int64_t dst_len, float *RESTRICT src, int64_t src_len) {
    if (LIKELY(src_len >= dst_len)) {                           /* [OPT-2] entry test */
        for (int64_t k = 0; k < dst_len; k++) {
            dst[k] = dst[k] * 0.5f + src[k] * 0.25f;
            src[k] = src[k] * 0.75f + 0.125f;
        }
    } else {                                                    /* [OPT-2] checked body */
        for (int64_t k = 0; k < dst_len; k++) {
            if (k >= src_len) e_panic("index out of bounds");  /* [TYP-31] src[k] */
            dst[k] = dst[k] * 0.5f + src[k] * 0.25f;
            src[k] = src[k] * 0.75f + 0.125f;
        }
    }
}

/* The copy with one name ([BRW-11] exception 1): `dst` and `src` are `s`; one name, noalias. */
static NOINLINE void blend__one(float *RESTRICT s, int64_t s_len) {
    for (int64_t k = 0; k < s_len; k++) {
        s[k] = s[k] * 0.5f + s[k] * 0.25f;
        s[k] = s[k] * 0.75f + 0.125f;
    }
}

#ifdef GENERIC_COPY
/* salvage_samevar_generic.c: one copy for every grouping of names, compiled with no noalias. */
static NOINLINE void blend__any(float *dst, int64_t dst_len, float *src, int64_t src_len) {
    if (LIKELY(src_len >= dst_len)) {                           /* [OPT-2] entry test */
        for (int64_t k = 0; k < dst_len; k++) {
            dst[k] = dst[k] * 0.5f + src[k] * 0.25f;
            src[k] = src[k] * 0.75f + 0.125f;
        }
    } else {
        for (int64_t k = 0; k < dst_len; k++) {
            if (k >= src_len) e_panic("index out of bounds");  /* [TYP-31] */
            dst[k] = dst[k] * 0.5f + src[k] * 0.25f;
            src[k] = src[k] * 0.75f + 0.125f;
        }
    }
}
#endif

static float left[POOL], right[POOL];
static volatile int64_t span_len = SPAN;   /* the length is not known when compiling */

static FORCEINLINE float *span_of(float *list, int64_t list_len, int64_t a, int64_t n) {
    if (UNLIKELY(a < 0 || n < 0 || n > list_len - a)) e_panic("range out of bounds");   /* [TYP-31] */
    return list + a;
}

int main(int argc, char **argv) {
    h_start();
    int one = argc > 1 && strcmp(argv[1], "one") == 0;
    int64_t n = span_len;
    h_seed(31);
    for (int k = 0; k < POOL; k++) { left[k] = (float)(h_rand() % 1000) * 0.001f; right[k] = (float)(h_rand() % 1000) * 0.001f; }
    for (int r = 0; r < ROUNDS; r++) {
        for (int j = 0; j < 256; j++) {
            if (one) {
                float *s = span_of(left, POOL, (int64_t)j * SPAN, n);
#ifdef GENERIC_COPY
                blend__any(s, n, s, n);
#else
                blend__one(s, n);
#endif
            } else {
                float *d = span_of(left, POOL, (int64_t)j * SPAN, n);
                float *s = span_of(right, POOL, (int64_t)((j * 97 + r) & 255) * SPAN, n);
                blend(d, n, s, n);
            }
        }
    }
    uint64_t s = 0;
    for (int k = 0; k < POOL; k++) s += (uint64_t)(int64_t)(left[k] * 1000000.0f) + (uint64_t)(int64_t)(right[k] * 1000000.0f) * 3u;
#ifndef NAME
#define NAME "model_samevar"
#endif
    char name[64]; snprintf(name, sizeof name, "%s-%s", NAME, one ? "one" : "two");
    h_report(name, s);
    return 0;
}
