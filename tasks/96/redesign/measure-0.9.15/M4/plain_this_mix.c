/* plain_this_mix.c: workload this_mix, plain C. A method that mixes one float list its object owns
   into another (`out[i] = out[i] * 0.5 + inp[i] * gain`) and tracks the peak of the mixed-in signal
   in a field. The element stores (`double`) are the same type as the object's `gain` and `peak`, so
   without `restrict` a store to `out[i]` may change them for the C compiler.

   What a careful C programmer writes: `void mixer_process(Mixer *m)`, the two lists' words read once,
   the lengths assumed equal. No checks. -DNOALIAS_THIS puts `restrict` on `m`. */
#include "m4.h"

#ifdef NOALIAS_THIS
#define THIS RESTRICT
#define NAME "plain_this_mix_noalias"
#else
#define THIS
#define NAME "plain_this_mix"
#endif

#define LEN 65536
#define FRAMES 3000

typedef struct { ArrF64 inp, out; double gain, peak; } Mixer;

static NOINLINE void mixer_process(Mixer *THIS m) {
    double *in = m->inp.ptr, *out = m->out.ptr;
    int64_t n = m->out.len;
    for (int64_t i = 0; i < n; i++) {
        double v = in[i] * m->gain;
        out[i] = out[i] * 0.5 + v;
        if (v > m->peak) m->peak = v;
    }
}

int main(void) {
    h_start();
    Mixer *m = (Mixer *)launder(calloc(1, sizeof(Mixer)));
    m->inp.ptr = (double *)malloc(LEN * sizeof(double)); m->inp.len = m->inp.cap = LEN;
    m->out.ptr = (double *)calloc(LEN, sizeof(double)); m->out.len = m->out.cap = LEN;
    h_seed(29);
    for (int i = 0; i < LEN; i++) m->inp.ptr[i] = (double)((int)(h_rand() % 2001) - 1000) * 0.001;
    for (int f = 0; f < FRAMES; f++) {
        m->gain = 0.25 * (double)(1 + f % 4);
        mixer_process(m);
    }
    uint64_t sum = (uint64_t)(int64_t)(m->peak * 1e6);
    for (int i = 0; i < LEN; i++) sum += (uint64_t)(int64_t)(m->out.ptr[i] * 1e6);
    h_report(NAME, sum);
    return 0;
}
