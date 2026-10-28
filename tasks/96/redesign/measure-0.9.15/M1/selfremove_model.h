/* selfremove_model.h: run 3's W4 (callbacks that remove themselves) as spec 0.9.15_Hardened_1 compiles
   it, repeated REPS times in one process so the run is long enough to time (one repetition is about
   25 ms). Source: target/redesign3/25-c-writer-fix/w4.c built -DSTATE_BLOCK (run 3's
   `model_w4_state_block`, the layout 0.9.15 keeps: R12 "a variable a function keeps with `owns`, in its
   own state"), with run 3's GCC-only check macros rewritten as portable functions (m1rt.h).

   The Ember program (w4.em in 0.9.15's words):
       class Dispatcher:
           handlers: Array[fn(ref mut Dispatcher, int, int) -> int] = []
           current: int = 0
           ...
       fn make_handler(m: int) -> fn(ref mut Dispatcher, int, int) -> int:   # local to main
           counter = 0
           return fn(ref mut d: Dispatcher, k: int, v: int) -> int owns counter, copy of m:
               if v >= 0:
                   counter += v * m
                   total += 1                     # main's `total`, the real variable, at its address
                   if k % 100_000 == 99_999:
                       d.unregister_current()     # removes this very handler: may end its own state
                       d.register(make_handler(k % 17 + 1))
               return counter                     # ... and uses the state after that call
   10,000 handlers, 10,000,000 dispatches per repetition; 100 of them remove themselves.

   What the model runs for the memory model, with the rule for each:
   - [M1] dispatch copies {code, state} out of the list and calls it; no count, no pointer into the
     list kept (R12: "a call through a list element or a field copies the two words out").
   - [M2] THE ROW MEASURED HERE: the handler's body holds one count on its own state across
     `d.unregister_current()`, which may end it, because the body uses its state after that call
     (R12: "A body that may end its own state (it removes itself) and uses that state afterwards holds
     one count on it across that call"; AN-60). The +1 is compared with the maximum first ([RT-7], R9);
     the -1 after the last use ends and frees the state.
   - [M3] removing a function value from the list ends it: one -1 of its state's count; at zero the
     state goes back to the slab (R12: "Replacing or ending a function value ends what its state owns").
   - [M4] making a handler: one state block from the slab (its `owns counter` and `copy of m` inside).
   Language checks kept, as run 3's model kept them (not this row's): the zero divisor of
   `k % len(self.handlers)`, the bounds checks of `self.handlers[self.current]` and of
   `remove(self.current)`, integer overflow on `+` and `*` where no range proves it.

   HOIST (salvage_selfremove_hoist.c): no [M2]. Every use of the state after the call is a read of a
   field the call cannot change (worked out from the call's code, R5: unregister_current, register and
   make_handler change `d.handlers`, never this state's fields), so the reads move above the call and
   the state may end inside it. Safety argument in the report. */
#pragma once
#include "m1rt.h"

#define H 10000
#define E 10000000
#define REPS 8

typedef struct Dispatcher Dispatcher;
typedef struct State { uint32_t rc, type; int64_t counter, m; int64_t *total; } State;   /* 32 bytes */
typedef int64_t (*HandlerFn)(State *st, Dispatcher *d, int64_t k, int64_t v);
typedef struct { HandlerFn fn; State *st; } FnValue;                                     /* {code, state} */
typedef struct { FnValue *ptr; int64_t len, cap; } FnArr;                                /* Ember's Array */
struct Dispatcher { uint32_t rc, type; FnArr handlers; int64_t current; };              /* a class object */

static Slab state_slab = {0, 0, 0, sizeof(State)};

static NOINLINE void fnarr_grow(FnArr *a) {
    int64_t nc = a->cap < 4 ? 4 : a->cap * 2;
    FnValue *p = (FnValue *)realloc(a->ptr, (size_t)nc * sizeof(FnValue));
    if (!p) e_stop("out of memory");
    a->ptr = p; a->cap = nc;
}

#ifndef HOIST
static inline void count_add(uint32_t *rc) {
    if (UNLIKELY(*rc == 0xFFFFFFFFu)) e_stop("too many owners of a function's state (4,294,967,295)");  /* [RT-7] */
    *rc += 1;
}
#endif
static inline void state_release(State *st) {                                /* [M3] */
    if (--st->rc == 0) slab_free(&state_slab, st);
}

static void unregister_current(Dispatcher *d) {                              /* self.handlers.remove(self.current) */
    int64_t i = d->current;
    if (UNLIKELY((uint64_t)i >= (uint64_t)d->handlers.len)) e_stop("index out of bounds");   /* bounds, as today */
    FnValue gone = d->handlers.ptr[i];
    memmove(&d->handlers.ptr[i], &d->handlers.ptr[i + 1], sizeof(FnValue) * (size_t)(d->handlers.len - i - 1));
    d->handlers.len--;
    state_release(gone.st);                                                  /* the removed value ends [M3] */
}
static inline void register_handler(Dispatcher *d, FnValue h) {
    if (d->handlers.len == d->handlers.cap) fnarr_grow(&d->handlers);
    d->handlers.ptr[d->handlers.len++] = h;
}

static int64_t handler(State *st, Dispatcher *d, int64_t k, int64_t v);
static FnValue make_handler(int64_t m, int64_t *total) {                     /* [M4] */
    State *st = (State *)slab_alloc(&state_slab);
    st->rc = 1; st->type = 0; st->counter = 0; st->m = m; st->total = total;
    FnValue f = {handler, st};
    return f;
}

static int64_t handler(State *st, Dispatcher *__restrict d, int64_t k, int64_t v) {   /* ref mut d: noalias (R4) */
    if (v >= 0) {
        st->counter = ck_add(st->counter, ck_mul(v, st->m));
        *st->total = ck_add(*st->total, 1);
        if (fmod_i64(k, 100000) == 99999) {
#ifdef HOIST
            int64_t c = st->counter;                                         /* salvage: read before the call */
            int64_t *total = st->total;
            unregister_current(d);                                           /* the state ends in here */
            register_handler(d, make_handler(fmod_i64(k, 17) + 1, total));
            return c;
#else
            count_add(&st->rc);                                              /* [M2] hold the state across the call */
            unregister_current(d);
            register_handler(d, make_handler(fmod_i64(k, 17) + 1, st->total));
            int64_t c = st->counter;
            state_release(st);                                               /* [M2] the hold ends: the state is freed */
            return c;
#endif
        }
    }
    return st->counter;
}

static inline void dispatch(Dispatcher *__restrict d, int64_t k) {             /* mut self: noalias (R4) */
    int64_t n = d->handlers.len;
    if (UNLIKELY(n == 0)) e_stop("division by zero");                        /* k % len(self.handlers) */
    d->current = fmod_i64(k, n);
    if (UNLIKELY((uint64_t)d->current >= (uint64_t)d->handlers.len)) e_stop("index out of bounds");
    FnValue c = d->handlers.ptr[d->current];                                 /* [M1] two words copied out */
    c.fn(c.st, d, k, fmod_i64(k, 1000));
}

static int64_t one_rep(void) {
    int64_t total = 0;
    Dispatcher *d = (Dispatcher *)malloc(sizeof(Dispatcher));               /* d = Dispatcher() */
    if (!d) e_stop("out of memory");
    d->rc = 1; d->type = 0; d->handlers.ptr = NULL; d->handlers.len = d->handlers.cap = 0; d->current = 0;
    for (int64_t i = 0; i < H; i++) register_handler(d, make_handler(i % 13 + 1, &total));
    for (int64_t k = 0; k < E; k++) dispatch(d, k);
    int64_t s = 0;
    for (int64_t i = 0; i < d->handlers.len; i++) {                         /* while i < len(d.handlers) */
        FnValue c = d->handlers.ptr[i];
        s = ck_add(s, c.fn(c.st, d, 0, -1));
    }
    int64_t out = ck_add(total, s);
    for (int64_t i = 0; i < d->handlers.len; i++) state_release(d->handlers.ptr[i].st);   /* d ends: its handlers end */
    free(d->handlers.ptr);
    free(d);
    return out;
}

int main(void) {
    h_start();
    uint64_t sum = 0;
    for (int rep = 0; rep < REPS; rep++) sum += (uint64_t)one_rep();
    h_report(NAME, sum);
    return 0;
}
