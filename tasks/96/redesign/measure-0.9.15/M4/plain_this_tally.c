/* plain_this_tally.c: workload this_tally, plain C. A method that walks a list its object owns and
   adds into other fields of the same object: a 256-bucket histogram (`hist[x & 255] += 1`), a running
   total and a running maximum. The element stores (`int64_t`) are the same type as the object's
   number fields, so without `restrict` the C compiler must assume a histogram store may change
   `total`, `max` or `hist.len` (type-based alias analysis cannot separate them).

   What a careful C programmer writes: `void stats_tally(Stats *s)`, fields used where the method
   uses them, the walked list's words read once. No checks. -DNOALIAS_THIS puts `restrict` on `s`. */
#include "m4.h"

#ifdef NOALIAS_THIS
#define THIS RESTRICT
#define NAME "plain_this_tally_noalias"
#else
#define THIS
#define NAME "plain_this_tally"
#endif

#define SAMPLES 65536
#define FRAMES 3000

typedef struct { ArrInt samples, hist; int64_t total, max, seen; } Stats;

static NOINLINE void stats_tally(Stats *THIS s) {
    int64_t *xs = s->samples.ptr;
    int64_t n = s->samples.len;
    for (int64_t i = 0; i < n; i++) {
        int64_t x = xs[i];
        s->hist.ptr[x & 255] += 1;
        s->total += x;
        if (x > s->max) s->max = x;
    }
    s->seen += n;
}

int main(void) {
    h_start();
    Stats *s = (Stats *)launder(calloc(1, sizeof(Stats)));
    s->samples.ptr = (int64_t *)malloc(SAMPLES * sizeof(int64_t));
    s->samples.len = s->samples.cap = SAMPLES;
    s->hist.ptr = (int64_t *)calloc(256, sizeof(int64_t));
    s->hist.len = s->hist.cap = 256;
    h_seed(23);
    for (int i = 0; i < SAMPLES; i++) s->samples.ptr[i] = (int64_t)(h_rand() % 1000000);
    for (int f = 0; f < FRAMES; f++) stats_tally(s);
    uint64_t sum = (uint64_t)s->total + (uint64_t)s->max + (uint64_t)s->seen;
    for (int k = 0; k < 256; k++) sum += (uint64_t)s->hist.ptr[k] * (uint64_t)(k + 1);
    h_report(NAME, sum);
    return 0;
}
