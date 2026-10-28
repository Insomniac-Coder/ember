/* salvage_respan_flag.c: workload respan, salvage "the callee says whether it re-pointed"
   (link with pkg_respan.c).

   Idea: keep the rule (a `ref mut` span may be re-pointed, so across a package's edge it is passed
   by its address), and let the callee return one flag in a register: 1 if any of its lines pointed
   `pos` at another span, else 0. The caller keeps what it knew of its span (start, length 16) and walks
   the chunk with it when the flag is 0; only when it is 1 does it read the span's words again and walk
   a loop of unknown length.

   Safety: the flag is written by the callee's own compiled code, every line that re-points `pos`
   (`pos = ...`) setting it, and every path through the body returning it; a callee compiled by
   another Ember compiler of the same version emits the same convention (it is part of the function's
   ABI, kept in the package's interface file, R17). When the flag is 1 the caller uses exactly what
   the model uses (the words read back). Nothing about which memory the span may view changes: R6's
   rule for what a callee may put there still decides it when compiling.
   -DRUNTIME_LEN as the model. Same checks as the model. */
#include "m4.h"

#define POOL 4096
#define CHUNK 16
#define FRAMES 117188

typedef struct { float *ptr; int64_t len; } MutSpanF;
int integrate_ref_flag(MutSpanF *RESTRICT pos, const float *vel, int64_t vel_len, float dt);

static float pos[POOL], vel[POOL];
#ifdef RUNTIME_LEN
static volatile int64_t chunk_len = CHUNK;
#define NAME "salvage_respan_flag_rtlen"
#else
#define NAME "salvage_respan_flag"
#endif

static FORCEINLINE float *span_of(float *list, int64_t list_len, int64_t a, int64_t n) {
    if (UNLIKELY(a < 0 || n < 0 || n > list_len - a)) e_panic("range out of bounds");   /* [TYP-31] */
    return list + a;
}

static FORCEINLINE void wrap(float *p, int64_t n) {
    for (int64_t i = 0; i < n; i++) {
        float x = p[i];
        if (x > 1000.0f) x -= 2000.0f; else if (x < -1000.0f) x += 2000.0f;
        p[i] = x;
    }
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
            float *p = span_of(pos, POOL, c * len, len);
            MutSpanF s;
            s.ptr = p; s.len = len;
            if (UNLIKELY(integrate_ref_flag(&s, v, len, dt))) wrap(s.ptr, s.len);   /* re-pointed: read back */
            else wrap(p, len);                                                       /* not: the caller's own words */
        }
    }
    uint64_t sum = 0;
    for (int i = 0; i < POOL; i++) sum += (uint64_t)(int64_t)(pos[i] * 1000.0f);
    h_report(NAME, sum);
    return 0;
}
