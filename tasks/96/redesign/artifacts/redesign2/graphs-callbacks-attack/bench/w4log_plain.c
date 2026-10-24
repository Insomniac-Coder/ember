/* W4 with a log callback, plain C (graphs-callbacks/bench/w4/w4_plain.c -DSLAB with the log call
   added, and with the `v >= 0` test the Python program has, so both sides do the same work).
   No checks, no counts: the C programmer knows the log cannot remove the handler. */
#include "harness.h"
#include <string.h>
#define H 10000
#define E 10000000

typedef struct Dispatcher Dispatcher;
typedef struct { int64_t (*fn)(void *env, Dispatcher *d, int64_t k, int64_t v); void *env; } Closure;
typedef struct { int64_t logged; } LogEnv;
typedef struct { void (*fn)(LogEnv *env, int64_t k); LogEnv *env; } LogClosure;
struct Dispatcher { Closure *hs; size_t n, cap, cur; LogClosure log; };
typedef struct { int64_t counter, m; int64_t *total; } Env;
static char *slab_cur, *slab_end; static Env *slab_free;
static Env *env_alloc(void) {
    if (slab_free) { Env *e = slab_free; slab_free = *(Env **)e; return e; }
    if (slab_cur == slab_end) { size_t sz = (size_t)4096 * sizeof(Env); slab_cur = malloc(sz); slab_end = slab_cur + sz; }
    Env *e = (Env *)slab_cur; slab_cur += sizeof(Env); return e;
}
static void env_free(Env *e) { *(Env **)e = slab_free; slab_free = e; }

static __attribute__((noinline)) void log_a(LogEnv *e, int64_t k) { e->logged += k & 1; }
static __attribute__((noinline)) void log_b(LogEnv *e, int64_t k) { e->logged += k & 1; }

static int64_t on_event(void *env, Dispatcher *d, int64_t k, int64_t v);
static void reg(Dispatcher *d, int64_t m, int64_t *total) {
    Env *e = env_alloc();
    e->counter = 0; e->m = m; e->total = total;
    if (d->n == d->cap) { d->cap = d->cap ? d->cap * 2 : 16; d->hs = realloc(d->hs, sizeof(Closure) * d->cap); }
    d->hs[d->n++] = (Closure){on_event, e};
}
static int64_t on_event(void *env, Dispatcher *d, int64_t k, int64_t v) {
    Env *e = env;
    if (v >= 0) {
        d->log.fn(d->log.env, k);
        e->counter += v * e->m;
        *e->total += 1;
        if (k % 100000 == 99999) {
            size_t i = d->cur;
            memmove(&d->hs[i], &d->hs[i + 1], sizeof(Closure) * (d->n - i - 1));
            d->n--;
            reg(d, k % 17 + 1, e->total);
            int64_t c = e->counter;
            env_free(e);
            return c;
        }
    }
    return e->counter;
}

int main(int argc, char **argv) {
    (void)argv;
    h_start();
    int64_t total = 0;
    LogEnv le = {0};
    Dispatcher d = {0};
    d.log = (LogClosure){ argc > 100 ? log_b : log_a, &le };
    for (int64_t i = 0; i < H; i++) reg(&d, i % 13 + 1, &total);
    for (int64_t k = 0; k < E; k++) {
        d.cur = (size_t)(k % (int64_t)d.n);
        Closure c = d.hs[d.cur];
        c.fn(c.env, &d, k, k % 1000);
    }
    int64_t s = 0;
    for (size_t i = 0; i < d.n; i++) { Closure c = d.hs[i]; s += c.fn(c.env, &d, 0, -1); }
    h_report("w4log-plain-slab", (uint64_t)(total + s + le.logged));
    return 0;
}
