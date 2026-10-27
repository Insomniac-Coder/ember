/* salvage_this_tally_hoist.c: the model with the object's fields held in locals across the walk
   (the front end's per-loop alias fact; idea and safety as salvage_this_tick_hoist.c).

   Proof for this loop: the loop's only stores are to `hist`'s elements (a buffer `this` owns, never the
   object's block), `total` and `max`; nothing in it reshapes `hist` (no push, no assignment of the
   field), so `hist`'s pointer and length are invariant and read once; `total` and `max` are changed
   only by the loop's own lines and written back after it. No call in the loop but the panics, which
   end the process with no unwinding ([PAN-1]). Same checks as the model, same order of effects. */
#include "m4.h"

#define SAMPLES 65536
#define FRAMES 3000

typedef struct { uint32_t count, type; ArrInt samples, hist; int64_t total, max, seen; } Stats;   /* [OBJ-1] */

static NOINLINE void Stats_tally(Stats *this_) {
    int64_t *xs = this_->samples.ptr;                           /* R8 (c) */
    int64_t n = this_->samples.len;
    int64_t *hist = this_->hist.ptr, hist_len = this_->hist.len;    /* invariant in the loop (proved) */
    int64_t total = this_->total, max = this_->max;                 /* changed only here (proved) */
    for (int64_t i = 0; i < n; i++) {
        int64_t x = xs[i];
        int64_t k = x & 255;
        if (UNLIKELY((uint64_t)k >= (uint64_t)hist_len)) e_panic("index out of bounds");   /* [TYP-31] */
        hist[k] = ck_add(hist[k], 1);                           /* [TYP-8] */
        total = ck_add(total, x);                               /* [TYP-8] */
        if (x > max) max = x;
    }
    this_->total = total; this_->max = max;                     /* written back after the loop */
    this_->seen = ck_add(this_->seen, n);                       /* [TYP-8] */
}

int main(void) {
    h_start();
    Stats *s = (Stats *)launder(calloc(1, sizeof(Stats)));
    s->count = 1; s->type = 1;
    s->samples.ptr = (int64_t *)malloc(SAMPLES * sizeof(int64_t));
    s->samples.len = s->samples.cap = SAMPLES;
    s->hist.ptr = (int64_t *)calloc(256, sizeof(int64_t));
    s->hist.len = s->hist.cap = 256;
    h_seed(23);
    for (int i = 0; i < SAMPLES; i++) s->samples.ptr[i] = (int64_t)(h_rand() % 1000000);
    for (int f = 0; f < FRAMES; f++) Stats_tally(s);
    uint64_t sum = (uint64_t)s->total + (uint64_t)s->max + (uint64_t)s->seen;
    for (int k = 0; k < 256; k++) sum += (uint64_t)s->hist.ptr[k] * (uint64_t)(k + 1);
    h_report("salvage_this_tally_hoist", sum);
    return 0;
}
