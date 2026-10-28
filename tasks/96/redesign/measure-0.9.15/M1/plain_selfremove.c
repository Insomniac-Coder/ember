/* plain_selfremove.c: run 3's plain C baseline for W4 (target/redesign2/bench/w4.c, the `plain_w4` of
   RUN3/26-timer.md), unchanged but for the REPS loop (8 repetitions in one process, each freeing what it
   made, as the model does) and the printed name. No safety checks. A closure is (function pointer,
   environment pointer); each environment is malloc'd and holds the counter, the multiplier and a raw
   pointer to the shared total. A handler removes itself from inside its own call (memmove keeps the
   order), appends its replacement, then frees its own environment as its last act: plain C needs no
   count because the programmer orders the free last by hand. Checksum 279,575,944,000. */
#include "harness.h"
#include <string.h>
#define H 10000
#define E 10000000
#define REPS 8

typedef struct Dispatcher Dispatcher;
typedef struct { void (*fn)(void *env, Dispatcher *d, int64_t k, int64_t v); void *env; } Closure;
struct Dispatcher { Closure *hs; size_t n, cap, cur; };
typedef struct { int64_t counter, m; int64_t *total; } Env;

static void on_event(void *env, Dispatcher *d, int64_t k, int64_t v);
static void reg(Dispatcher *d, int64_t m, int64_t *total) {
    Env *e = malloc(sizeof(Env));
    e->counter = 0; e->m = m; e->total = total;
    if (d->n == d->cap) { d->cap = d->cap ? d->cap * 2 : 16; d->hs = realloc(d->hs, sizeof(Closure) * d->cap); }
    d->hs[d->n++] = (Closure){on_event, e};
}
static void on_event(void *env, Dispatcher *d, int64_t k, int64_t v) {
    Env *e = env;
    e->counter += v * e->m;
    *e->total += 1;
    if (k % 100000 == 99999) {
        size_t i = d->cur;
        memmove(&d->hs[i], &d->hs[i + 1], sizeof(Closure) * (d->n - i - 1));
        d->n--;
        reg(d, k % 17 + 1, e->total);
        free(e);
    }
}

static int64_t one_rep(void) {
    int64_t total = 0;
    Dispatcher d = {0};
    for (int64_t i = 0; i < H; i++) reg(&d, i % 13 + 1, &total);
    for (int64_t k = 0; k < E; k++) {
        d.cur = (size_t)(k % (int64_t)d.n);
        Closure c = d.hs[d.cur];
        c.fn(c.env, &d, k, k % 1000);
    }
    int64_t s = 0;
    for (size_t i = 0; i < d.n; i++) s += ((Env *)d.hs[i].env)->counter;
    for (size_t i = 0; i < d.n; i++) free(d.hs[i].env);
    free(d.hs);
    return total + s;
}

int main(void) {
    h_start();
    uint64_t sum = 0;
    for (int rep = 0; rep < REPS; rep++) sum += (uint64_t)one_rep();
    h_report("plain-selfremove", sum);
    return 0;
}
