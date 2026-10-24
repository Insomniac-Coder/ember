/* combined-w4: W4 (callbacks) as the combined memory model's compiler would emit it.
   Report: tasks/96/redesign/phase3b-combined.md, section 6.4. Built from the graphs group's
   target/redesign2/graphs-callbacks/bench/w4/w4_ember_point.c; the Python twin of this program is
   target/redesign2/graphs-callbacks/python/w4.py.

   The Ember program (the model's syntax):

       class Dispatcher:
           handlers: Array[fn(Dispatcher, int, int) -> int] = []
           current: int = 0
           fn register(self, h: fn(Dispatcher, int, int) -> int): self.handlers.push(h)
           fn unregister_current(self): self.handlers.remove(self.current)
           fn dispatch(self, k: int):
               self.current = k % len(self.handlers)
               self.handlers[self.current](self, k, k % 1000)

       fn main():
           total = 0
           fn make_handler(m: int) -> fn(Dispatcher, int, int) -> int:
               counter = 0
               fn handler(d: Dispatcher, k: int, v: int) -> int:
                   if v >= 0:
                       counter += v * m
                       total += 1
                       if k % 100_000 == 99_999:
                           d.unregister_current()                 # removes this handler, from inside its call
                           d.register(make_handler(k % 17 + 1))
                   return counter
               return handler
           ...

   What the model emits, and nothing else:
   - A closure stored in a list keeps its captured state in a box (A32): {count, counter, m, &total},
     32 bytes, from the slab for its size. A box's class is final with no base: a 4-byte header.
     The list element is {function, box}, 16 bytes.
   - `total` is captured by pointer: every closure that captures it is stored only in `d`, a local
     declared after `total`, so they all die before it (A12's proof; the references group's limits).
     `counter` lives in the box; `m` is copied into it.
   - dispatch copies {function, box} out of the slot and calls it; it uses nothing of it after the
     call, so it protects nothing: no count, no flag, no copy back. No bounds check: `k % len` with a
     positive length is below the length (floor modulo), and nothing changes the list in between.
   - The handler body uses its box after `d.unregister_current()`, a call whose effect summary may
     remove a closure stored in Dispatcher.handlers, so it retains its own box across that call and
     releases it after (once per 100,000 events). The box is freed there.
   - `remove` checks its position; the retain checks the count for overflow.
   - Dispatcher is a final class with no base: a 4-byte header.
   - Integer overflow checks left out, as in every model.
   Self-check build: -DPOISON marks freed boxes, never reuses them, and checks every use. */
#include "../../../redesign/bench/harness.h"   /* path changed when the file moved into first-run/ */
#include <string.h>
#define H 10000
#define E 10000000

static __declspec(noreturn) void panic(const char *m) { fprintf(stderr, "panic: %s\n", m); exit(101); }
#define CHECK(c, m) do { if (__builtin_expect(!(c), 0)) panic(m); } while (0)

typedef struct Dispatcher Dispatcher;
typedef struct State { uint32_t rc; int64_t counter, m; int64_t *total; } State;   /* the box */
_Static_assert(sizeof(State) == 32, "box size");
typedef int64_t (*HandlerFn)(State *st, Dispatcher *d, int64_t k, int64_t v);
typedef struct { HandlerFn fn; State *st; } Callable;                               /* 16 bytes */
typedef struct { Callable *ptr; size_t len, cap; } CVec;
struct Dispatcher { uint32_t rc; CVec hs; int64_t current; };

/* the slab for 32-byte objects: carved in order, then a free list */
static char *slab_cur, *slab_end; static State *slab_free;
static State *state_alloc(void) {
    if (slab_free) { State *s = slab_free; slab_free = *(State **)s; return s; }
    if (slab_cur == slab_end) {
        size_t sz = (size_t)4096 * sizeof(State);
        slab_cur = malloc(sz); CHECK(slab_cur, "out of memory");
        slab_end = slab_cur + sz;
    }
    State *s = (State *)slab_cur; slab_cur += sizeof(State); return s;
}
static inline void state_retain(State *s) { CHECK(s->rc - 1u < 0x0FFFFFFEu, "count overflow"); s->rc++; }
static inline void state_release(State *s) {
    if (--s->rc != 0) return;
#ifdef POISON   /* self-check build: a freed box is marked and never reused */
    s->m = -999; return;
#endif
    *(State **)s = slab_free; slab_free = s;
}

static void cvec_push(CVec *v, Callable c) {
    if (v->len == v->cap) {
        size_t cap = v->cap < 2 ? 4 : v->cap * 2;
        v->ptr = realloc(v->ptr, cap * sizeof(Callable)); CHECK(v->ptr, "out of memory");
        v->cap = cap;
    }
    v->ptr[v->len++] = c;
}
static void unregister_current(Dispatcher *d) {          /* self.handlers.remove(self.current) */
    size_t i = (size_t)d->current;
    CHECK(i < d->hs.len, "index out of bounds");
    State *gone = d->hs.ptr[i].st;
    memmove(&d->hs.ptr[i], &d->hs.ptr[i + 1], sizeof(Callable) * (d->hs.len - i - 1));
    d->hs.len--;
    state_release(gone);                                  /* the removed element is dropped */
}
static void register_handler(Dispatcher *d, Callable c) { cvec_push(&d->hs, c); }
static int64_t handler(State *st, Dispatcher *d, int64_t k, int64_t v);
static Callable make_handler(int64_t m, int64_t *total) {
    State *s = state_alloc();
    s->rc = 1; s->counter = 0; s->m = m; s->total = total;
    return (Callable){handler, s};
}

static int64_t handler(State *st, Dispatcher *d, int64_t k, int64_t v) {
#ifdef POISON
    CHECK(st->m != -999, "poison: a freed box was used");
#endif
    if (v >= 0) {
        st->counter += v * st->m;
        *st->total += 1;
        if (k % 100000 == 99999) {
            state_retain(st);                            /* the box is used after a call that may release it */
            unregister_current(d);
            register_handler(d, make_handler(k % 17 + 1, st->total));
            int64_t c = st->counter;
#ifdef POISON
            CHECK(st->m != -999, "poison: the box was freed before its last use");
#endif
            state_release(st);                           /* freed here: the list no longer holds it */
            return c;
        }
    }
    return st->counter;
}

static inline void dispatch(Dispatcher *d, int64_t k) {
    d->current = k % (int64_t)d->hs.len;
    Callable c = d->hs.ptr[d->current];                   /* copied out; not used after the call */
    c.fn(c.st, d, k, k % 1000);
}

int main(void) {
    h_start();
    int64_t total = 0;
    Dispatcher *d = malloc(sizeof(Dispatcher));          /* a class object: count 1 */
    CHECK(d, "out of memory");
    d->rc = 1; d->hs = (CVec){0}; d->current = 0;
    for (int64_t i = 0; i < H; i++) register_handler(d, make_handler(i % 13 + 1, &total));
    for (int64_t k = 0; k < E; k++) dispatch(d, k);
    int64_t s = 0;
    for (size_t i = 0; i < d->hs.len; i++) { Callable c = d->hs.ptr[i]; s += c.fn(c.st, d, 0, -1); }
#ifdef POISON
    puts("poison check ok: no box used after it was freed");
#endif
    h_report("combined-w4", (uint64_t)(total + s));
    return 0;
}
