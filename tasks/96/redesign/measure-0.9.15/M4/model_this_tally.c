/* model_this_tally.c: workload this_tally, what Ember emits under the rule as written.

       class Stats:
           samples: Array[int]
           hist: Array[int]          # 256 buckets, made with 256 zeros
           total: int = 0
           max: int = 0
           seen: int = 0

           fn tally():
               for x in this.samples:
                   this.hist[x & 255] += 1
                   this.total += x
                   if x > this.max:
                       this.max = x
               this.seen += this.samples.len()

   Accepted: the walk is over `samples`; the loop changes `hist`, `total` and `max`, other fields
   (R8 (c), [BRW-4] disjoint fields). `tally` changes its object, so `this` is `Stats *` with no
   `restrict` (verdict 18; [BRW-22], [EXC-15], [SIMD-3]); -DNOALIAS_THIS adds it (used only by
   salvage_this_tally_typeproof.c).

   Checks kept, each tagged: [TYP-8] overflow of each `+=`; [TYP-31] bounds of `hist[x & 255]`: the
   index is proved in 0..255 (a range fact, [RNG-4]), but `hist` is an `Array` whose length is not known
   when compiling, so the test of the index against `hist.len` stays (it is read where it is used: the
   loop does not reshape `hist`, but `hist.len` is a field the C compiler sees as `int64_t` memory).
   The walk reads `samples`' words once (R8 (c)). */
#include "m4.h"

#ifdef NOALIAS_THIS
#define THIS RESTRICT
#ifndef NAME
#define NAME "model_this_tally_noalias"
#endif
#else
#define THIS
#define NAME "model_this_tally"
#endif

#define SAMPLES 65536
#define FRAMES 3000

/* [OBJ-1]: a class object, 8-byte header */
typedef struct { uint32_t count, type; ArrInt samples, hist; int64_t total, max, seen; } Stats;

static NOINLINE void Stats_tally(Stats *THIS this_) {
    int64_t *xs = this_->samples.ptr;                           /* R8 (c): the walked list's words, once */
    int64_t n = this_->samples.len;
    for (int64_t i = 0; i < n; i++) {
        int64_t x = xs[i];
        int64_t k = x & 255;
        if (UNLIKELY((uint64_t)k >= (uint64_t)this_->hist.len)) e_panic("index out of bounds");   /* [TYP-31] */
        this_->hist.ptr[k] = ck_add(this_->hist.ptr[k], 1);     /* [TYP-8] */
        this_->total = ck_add(this_->total, x);                 /* [TYP-8] */
        if (x > this_->max) this_->max = x;
    }
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
    h_report(NAME, sum);
    return 0;
}
