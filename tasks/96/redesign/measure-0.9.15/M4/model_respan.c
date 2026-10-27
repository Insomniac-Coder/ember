/* model_respan.c: workload respan, what Ember emits (link with pkg_respan.c).

       # the other package: pub fn integrate(ref mut pos: MutSpan[f32], vel: Span[f32], dt: f32)
       for f in 0..FRAMES:
           for c in 0..256:
               s = ref mut pos[c * len .. c * len + len].as_mut_span()
               integrate(ref mut s, vel[c * len .. c * len + len].as_span(), 0.01)
               for ref mut x in s:
                   if x > 1000.0: x -= 2000.0
                   elif x < -1000.0: x += 2000.0

   Default: the rule as written for a call across a package's edge: `ref mut pos` may point the
   caller's span elsewhere (R4; run4/v5/area-A.md near line 2507), so `s` lives in memory, the call gets
   its address (`integrate_ref`), and after the call the caller reads `s`'s pointer and length again:
   what it knew of them (its start, its length 16) is gone, so the walk after the call is a loop of
   unknown length.
   -DWORDS: the same program with `integrate` declared `const ref mut pos` (verdict 42; the call then
   writes `integrate(const ref mut s, ...)`), or called inside its own package: the span goes as its two
   words (`integrate_words`) and the caller keeps them.
   -DRUNTIME_LEN: the chunk length read from a volatile, not known when compiling (as plain C's).

   Checks kept: [TYP-31] making each span checks its range against the list's length; the callee's
   [OPT-2] entry test (in pkg_respan.c). The walk after the call is by element: no checks. */
#include "m4.h"

#define POOL 4096
#define CHUNK 16
#define FRAMES 117188

typedef struct { float *ptr; int64_t len; } MutSpanF;
void integrate_words(float *RESTRICT pos, int64_t pos_len, const float *vel, int64_t vel_len, float dt);
void integrate_ref(MutSpanF *RESTRICT pos, const float *vel, int64_t vel_len, float dt);

static float pos[POOL], vel[POOL];
#ifdef RUNTIME_LEN
static volatile int64_t chunk_len = CHUNK;
#endif
#if defined(WORDS) && defined(RUNTIME_LEN)
#define NAME "model_respan_words_rtlen"
#elif defined(WORDS)
#define NAME "model_respan_words"
#elif defined(RUNTIME_LEN)
#define NAME "model_respan_rtlen"
#else
#define NAME "model_respan"
#endif

static FORCEINLINE float *span_of(float *list, int64_t list_len, int64_t a, int64_t n) {
    if (UNLIKELY(a < 0 || n < 0 || n > list_len - a)) e_panic("range out of bounds");   /* [TYP-31] */
    return list + a;
}

int main(void) {
    h_start();
#ifdef RUNTIME_LEN
    int64_t len = chunk_len;
#else
    const int64_t len = CHUNK;
#endif
    h_seed(37);
    for (int i = 0; i < POOL; i++) { pos[i] = (float)((int)(h_rand() % 2001) - 1000); vel[i] = (float)((int)(h_rand() % 2001) - 1000) * 0.01f; }
    const float dt = 0.01f;
    for (int f = 0; f < FRAMES; f++) {
        for (int64_t c = 0; c < POOL / CHUNK; c++) {
            float *v = span_of(vel, POOL, c * len, len);
#ifdef WORDS
            float *p = span_of(pos, POOL, c * len, len);
            int64_t n = len;
            integrate_words(p, n, v, len, dt);                  /* the caller keeps p and n */
#else
            MutSpanF s;                                         /* `s` may be re-pointed: it lives in memory */
            s.ptr = span_of(pos, POOL, c * len, len);
            s.len = len;
            integrate_ref(&s, v, len, dt);
            float *p = s.ptr;                                   /* read again after the call */
            int64_t n = s.len;
#endif
            for (int64_t i = 0; i < n; i++) {                   /* `for ref mut x in s:` */
                float x = p[i];
                if (x > 1000.0f) x -= 2000.0f; else if (x < -1000.0f) x += 2000.0f;
                p[i] = x;
            }
        }
    }
    uint64_t sum = 0;
    for (int i = 0; i < POOL; i++) sum += (uint64_t)(int64_t)(pos[i] * 1000.0f);
    h_report(NAME, sum);
    return 0;
}
