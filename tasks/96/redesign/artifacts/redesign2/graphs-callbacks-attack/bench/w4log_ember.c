/* W4 with a log callback: the attack on A32's "the body retains its box across a call that may
   release it" when that call happens on EVERY event, not only on the rare path.

   The Python program (python/w4log.py prints the checksum every model prints):

       def handler(d, k, v):
           nonlocal counter, total
           if v >= 0:
               d.log(k)              # a stored callback: code this handler cannot see
               counter += v * m      # the handler's state is used after that call
               total += 1
               if k % 100_000 == 99_999:
                   d.unregister_current(); d.register(make_handler(k % 17 + 1))
           return counter

   This file is w4_ember_point.c (graphs-callbacks/bench/w4) with that one line added. Under the
   report's rule 3 the body retains its own box across `d.log(k)`: a call through a stored closure has
   every effect, so it may remove this handler and release its box, and the body uses the box after it.
   -DNORETAIN leaves the retain out (NOT safe: only to measure what it costs).
   The log closure is chosen at run time from two identical functions so clang cannot see through
   the indirect call (plain C and Rust do the same). */
#include "../../graphs-callbacks/bench/ember_model.h"
#define H 10000
#define E 10000000

typedef struct Dispatcher Dispatcher;
typedef struct State { uint32_t rc; int64_t counter, m; int64_t *total; } State;   /* 32 bytes */
typedef int64_t (*HandlerFn)(State *st, Dispatcher *d, int64_t k, int64_t v);
typedef struct { HandlerFn fn; State *st; } Callable;
typedef struct { Callable *ptr; size_t len, cap; } CVec;
typedef struct LogState { uint32_t rc; int64_t logged; } LogState;
typedef void (*LogFn)(LogState *st, int64_t k);
typedef struct { LogFn fn; LogState *st; } LogCallable;
struct Dispatcher { uint32_t rc, type; CVec hs; int64_t current; LogCallable log; };

static char *slab_cur, *slab_end; static State *slab_free;
static State *state_alloc(void) {
    if (slab_free) { State *s = slab_free; slab_free = *(State **)s; return s; }
    if (slab_cur == slab_end) { size_t sz = (size_t)4096 * sizeof(State); slab_cur = malloc(sz); slab_end = slab_cur + sz; }
    State *s = (State *)slab_cur; slab_cur += sizeof(State); return s;
}
static void state_release(State *s) {
    if (--s->rc == 0) { *(State **)s = slab_free; slab_free = s; }
}
static void cvec_push(CVec *v, Callable c) {
    if (v->len == v->cap) { size_t cap = v->cap < 2 ? 4 : v->cap * 2; v->ptr = realloc(v->ptr, cap * sizeof(Callable)); v->cap = cap; }
    v->ptr[v->len++] = c;
}
static void unregister_current(Dispatcher *d) {
    size_t i = (size_t)d->current;
    e_bounds(i, d->hs.len);
    State *gone = d->hs.ptr[i].st;
    memmove(&d->hs.ptr[i], &d->hs.ptr[i + 1], sizeof(Callable) * (d->hs.len - i - 1));
    d->hs.len--;
    state_release(gone);
}
static int64_t handler(State *st, Dispatcher *d, int64_t k, int64_t v);
static Callable make_handler(int64_t m, int64_t *total) {
    State *s = state_alloc();
    s->rc = 1; s->counter = 0; s->m = m; s->total = total;
    return (Callable){ handler, s };
}
static void register_handler(Dispatcher *d, Callable c) { cvec_push(&d->hs, c); }

static __attribute__((noinline)) void log_a(LogState *st, int64_t k) { st->logged += k & 1; }
static __attribute__((noinline)) void log_b(LogState *st, int64_t k) { st->logged += k & 1; }

static int64_t handler(State *st, Dispatcher *d, int64_t k, int64_t v) {
    if (v >= 0) {
#ifndef NORETAIN
        st->rc++;                                  /* used after d.log(k), which may release it */
#endif
        LogCallable lg = d->log;                   /* copied out of the field; not used after */
        lg.fn(lg.st, k);
        st->counter += v * st->m;
        *st->total += 1;
        if (k % 100000 == 99999) {
#ifdef NORETAIN
            st->rc++;                              /* the original rare-path retain */
#endif
            unregister_current(d);
            register_handler(d, make_handler(k % 17 + 1, st->total));
#ifdef NORETAIN
            int64_t c = st->counter; state_release(st); return c;
#endif
        }
#ifndef NORETAIN
        int64_t c = st->counter;
        state_release(st);
        return c;
#endif
    }
    return st->counter;
}

static inline void dispatch(Dispatcher *d, int64_t k) {
    d->current = k % (int64_t)d->hs.len;
    size_t i = (size_t)d->current;
    e_bounds(i, d->hs.len);
    Callable c = d->hs.ptr[i];
    c.fn(c.st, d, k, k % 1000);
}

int main(int argc, char **argv) {
    (void)argv;
    h_start();
    int64_t total_local = 0; int64_t *total = &total_local;
    LogState *ls = malloc(sizeof(LogState)); ls->rc = 1; ls->logged = 0;
    Dispatcher *d = malloc(sizeof(Dispatcher));
    d->rc = 1; d->type = 1; d->hs = (CVec){0}; d->current = 0;
    d->log = (LogCallable){ argc > 100 ? log_b : log_a, ls };
    for (int64_t i = 0; i < H; i++) register_handler(d, make_handler(i % 13 + 1, total));
    for (int64_t k = 0; k < E; k++) dispatch(d, k);
    int64_t s = 0;
    for (size_t i = 0; i < d->hs.len; i++) { Callable c = d->hs.ptr[i]; s += c.fn(c.st, d, 0, -1); }
#ifdef NORETAIN
    h_report("w4log-ember-noretain", (uint64_t)(total_local + s + ls->logged));
#else
    h_report("w4log-ember", (uint64_t)(total_local + s + ls->logged));
#endif
    return 0;
}
