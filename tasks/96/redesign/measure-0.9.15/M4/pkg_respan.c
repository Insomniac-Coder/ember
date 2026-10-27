/* pkg_respan.c: the other package of workload respan, compiled as its own object and linked with
   each caller (no link-time optimisation), so the caller cannot see these bodies: a call across a
   package's edge.

       pub fn integrate(ref mut pos: MutSpan[f32], vel: Span[f32], dt: f32):
           for i in 0..pos.len():
               pos[i] += vel[i] * dt

   `ref mut pos` may write the elements the caller's span views and may point that span elsewhere
   (R4; run4/v5/area-A.md near line 2507). Across a package's edge the caller cannot know the body
   never does, so the span is passed by its address (`integrate_ref`). Written `const ref mut pos`
   (verdict 42), or called from inside the package, where the compiler sees the body never re-points
   it, the span is passed as its two words (`integrate_words`). Each body is the same
   `integrate_body`: `pos`'s elements noalias ([BRW-22]: the helper's `restrict` parameter, which
   clang keeps as alias scopes when it inlines it; a `restrict` local would be ignored), the [OPT-2]
   entry test on `vel`'s length, the checked body ([TYP-31]) when it fails. `vel` is a plain `Span`,
   not noalias ([BRW-22] marks only `ref mut` parameters; `pos`'s noalias is enough for the loop). */
#include "m4.h"

typedef struct { float *ptr; int64_t len; } MutSpanF;

/* plain C: what a careful C programmer exports (pointer and length, `restrict` on the written one) */
void integrate_c(float *RESTRICT pos, const float *vel, int64_t n, float dt) {
    for (int64_t i = 0; i < n; i++) pos[i] += vel[i] * dt;
}

static FORCEINLINE void integrate_body(float *RESTRICT p, int64_t n, const float *v, int64_t vn, float dt) {
    if (LIKELY(vn >= n)) {                                      /* [OPT-2] entry test */
        for (int64_t i = 0; i < n; i++) p[i] += v[i] * dt;
    } else {                                                    /* [OPT-2] checked body */
        for (int64_t i = 0; i < n; i++) {
            if (i >= vn) e_panic("index out of bounds");        /* [TYP-31] vel[i] */
            p[i] += v[i] * dt;
        }
    }
}

/* `const ref mut pos` (verdict 42), or a call inside the package: the span as its two words */
void integrate_words(float *RESTRICT pos, int64_t pos_len, const float *vel, int64_t vel_len, float dt) {
    integrate_body(pos, pos_len, vel, vel_len, dt);
}

/* `ref mut pos` across a package's edge: the caller's span by its address (it may be re-pointed) */
void integrate_ref(MutSpanF *RESTRICT pos, const float *vel, int64_t vel_len, float dt) {
    integrate_body(pos->ptr, pos->len, vel, vel_len, dt);      /* the span's words read once, before the loop */
}

/* Salvage (salvage_respan_flag.c): `ref mut pos` by its address as the rule says, and the callee
   answers whether it pointed the span elsewhere. Each line that re-points `pos` sets the answer to 1;
   this body has none, so it answers 0. */
int integrate_ref_flag(MutSpanF *RESTRICT pos, const float *vel, int64_t vel_len, float dt) {
    integrate_body(pos->ptr, pos->len, vel, vel_len, dt);
    return 0;
}
