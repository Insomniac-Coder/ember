/* model_this_mix.c: workload this_mix, what Ember emits under the rule as written.

       class Mixer:
           inp: Array[float]
           out: Array[float]
           gain: float = 1.0
           peak: float = 0.0

           fn process():
               for i in 0..this.out.len():
                   v = this.inp[i] * this.gain
                   this.out[i] = this.out[i] * 0.5 + v
                   if v > this.peak:
                       this.peak = v

   `process` changes its object, so `this` is `Mixer *` with no `restrict` (verdict 18; [BRW-22],
   [EXC-15], [SIMD-3]); -DNOALIAS_THIS adds it (used only by salvage_this_mix_typeproof.c).

   Checks kept: [OPT-2] the counted loop indexes `out[i]` and `inp[i]` with bases and lengths invariant
   in the loop (nothing in it reshapes either list, R8 (b)), so the compiler emits one entry test that
   every index is in bounds (`inp.len >= out.len`; `out[i]` is in bounds by the loop's range), the
   unchecked body when it passes, and the checked body ([TYP-31]) otherwise; behaviour unchanged. The
   lists' words are read once for [OPT-2]. No integer arithmetic to check. */
#include "m4.h"

#ifdef NOALIAS_THIS
#define THIS RESTRICT
#ifndef NAME
#define NAME "model_this_mix_noalias"
#endif
#else
#define THIS
#define NAME "model_this_mix"
#endif

#define LEN 65536
#define FRAMES 3000

typedef struct { uint32_t count, type; ArrF64 inp, out; double gain, peak; } Mixer;   /* [OBJ-1] */

static NOINLINE void Mixer_process(Mixer *THIS this_) {
    double *in = this_->inp.ptr, *out = this_->out.ptr;         /* [OPT-2]: bases and lengths, read once */
    int64_t in_len = this_->inp.len, n = this_->out.len;
    if (LIKELY(in_len >= n)) {                                  /* [OPT-2] entry test */
        for (int64_t i = 0; i < n; i++) {
            double v = in[i] * this_->gain;
            out[i] = out[i] * 0.5 + v;
            if (v > this_->peak) this_->peak = v;
        }
    } else {                                                    /* [OPT-2] the checked body */
        for (int64_t i = 0; i < n; i++) {
            if ((uint64_t)i >= (uint64_t)in_len) e_panic("index out of bounds");   /* [TYP-31] inp[i] */
            double v = in[i] * this_->gain;
            out[i] = out[i] * 0.5 + v;
            if (v > this_->peak) this_->peak = v;
        }
    }
}

int main(void) {
    h_start();
    Mixer *m = (Mixer *)launder(calloc(1, sizeof(Mixer)));
    m->count = 1; m->type = 1;
    m->inp.ptr = (double *)malloc(LEN * sizeof(double)); m->inp.len = m->inp.cap = LEN;
    m->out.ptr = (double *)calloc(LEN, sizeof(double)); m->out.len = m->out.cap = LEN;
    h_seed(29);
    for (int i = 0; i < LEN; i++) m->inp.ptr[i] = (double)((int)(h_rand() % 2001) - 1000) * 0.001;
    for (int f = 0; f < FRAMES; f++) {
        m->gain = 0.25 * (double)(1 + f % 4);
        Mixer_process(m);
    }
    uint64_t sum = (uint64_t)(int64_t)(m->peak * 1e6);
    for (int i = 0; i < LEN; i++) sum += (uint64_t)(int64_t)(m->out.ptr[i] * 1e6);
    h_report(NAME, sum);
    return 0;
}
