/* salvage_this_mix_hoist.c: the model with `gain` and `peak` held in locals across the loop (the
   front end's per-loop alias fact; idea and safety as salvage_this_tick_hoist.c).

   Proof for this loop: its only stores are to `out`'s elements (a buffer the object owns, never its
   block) and to `peak`; nothing writes `gain`; `peak` is changed only by the loop's own line and is
   written back after the loop. The checked body's panic ends the process ([PAN-1]). Same checks as
   the model ([OPT-2] entry test, [TYP-31] in the checked body), same results. */
#include "m4.h"

#define LEN 65536
#define FRAMES 3000

typedef struct { uint32_t count, type; ArrF64 inp, out; double gain, peak; } Mixer;   /* [OBJ-1] */

static NOINLINE void Mixer_process(Mixer *this_) {
    double *in = this_->inp.ptr, *out = this_->out.ptr;         /* [OPT-2] */
    int64_t in_len = this_->inp.len, n = this_->out.len;
    double gain = this_->gain, peak = this_->peak;              /* read only / changed only here (proved) */
    if (LIKELY(in_len >= n)) {                                  /* [OPT-2] entry test */
        for (int64_t i = 0; i < n; i++) {
            double v = in[i] * gain;
            out[i] = out[i] * 0.5 + v;
            if (v > peak) peak = v;
        }
    } else {
        for (int64_t i = 0; i < n; i++) {
            if ((uint64_t)i >= (uint64_t)in_len) e_panic("index out of bounds");   /* [TYP-31] */
            double v = in[i] * gain;
            out[i] = out[i] * 0.5 + v;
            if (v > peak) peak = v;
        }
    }
    this_->peak = peak;                                         /* written back after the loop */
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
    h_report("salvage_this_mix_hoist", sum);
    return 0;
}
