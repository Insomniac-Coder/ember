/* W4, Ember: the same program as w4_ember.c, under the rule this report chooses for A12 and A32:
   "a stored closure is an object; it is protected where it is used, not where it is called".

   - A closure value that is stored in an Array or a field keeps its captured state in a box: a small
     counted object (count + state), from the runtime's slab for its size (32 bytes here: count,
     counter, m, &total). The element is {function, box}, 16 bytes. This is the representation plain
     C (`env`) and Rust (Box<dyn FnMut>) use. A closure in a local keeps [CLO-10]'s inline state.
   - Calling a stored closure copies {function, box} out of the slot and calls it. The caller uses
     nothing of it after the call, so it protects nothing: no count, no mark, no copy back.
   - The closure's body is compiled like a method of its box. The ownership group's rule (a handle used
     after a call that may release it is retained across that call; phase3b-ownership.md 2.1 and 2.3)
     applies to the box as to any object: `d.unregister_current()` may remove a closure stored in
     `Dispatcher.handlers` (field effects say so), and the body uses its box after it, so the body
     retains its box across that call (its rare path) and releases it after; the box is freed there
     if the removal dropped the last other count.
   - This is the salvage report's "protect at the point of use" (phase3b-salvage.md, its W4 rows and
     target/redesign2/salvage/d3/w4.c), written with a count instead of a pin.
   With -DBOXED, `total` is itself in a counted box (what the rules emit when the proof that the
   closures die before `total` is not available). */
#include "../ember_model.h"
#define H 10000
#define E 10000000

typedef struct Dispatcher Dispatcher;
#ifdef BOXED
typedef struct { uint32_t rc; int64_t value; } Box64;
#define TOTAL_T Box64 *
#define TOTAL_ADD(t) ((t)->value += 1)
#define TOTAL_RETAIN(t) ((t)->rc++)
#define TOTAL_RELEASE(t) do { if (--(t)->rc == 0) free(t); } while (0)
#else
#define TOTAL_T int64_t *
#define TOTAL_ADD(t) (*(t) += 1)
#define TOTAL_RETAIN(t) ((void)0)
#define TOTAL_RELEASE(t) ((void)0)
#endif
typedef struct State { uint32_t rc; int64_t counter, m; TOTAL_T total; } State;   /* 32 bytes */
_Static_assert(sizeof(State) == 32, "box size");
typedef int64_t (*HandlerFn)(State *st, Dispatcher *d, int64_t k, int64_t v);
typedef struct { HandlerFn fn; State *st; } Callable;                             /* 16 bytes */
typedef struct { Callable *ptr; size_t len, cap; } CVec;
struct Dispatcher { uint32_t rc, type; CVec hs; int64_t current; };

/* the runtime's slab for 32-byte objects: bump, then a free list */
static char *slab_cur, *slab_end; static State *slab_free;
static State *state_alloc(void) {
    if (slab_free) { State *s = slab_free; slab_free = *(State **)s; return s; }
    if (slab_cur == slab_end) { size_t sz = (size_t)4096 * sizeof(State); slab_cur = malloc(sz); slab_end = slab_cur + sz; }
    State *s = (State *)slab_cur; slab_cur += sizeof(State); return s;
}
static void state_release(State *s) {
    if (--s->rc == 0) { TOTAL_RELEASE(s->total); *(State **)s = slab_free; slab_free = s; }
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
static Callable make_handler(int64_t m, TOTAL_T total) {
    TOTAL_RETAIN(total);
    State *s = state_alloc();
    s->rc = 1; s->counter = 0; s->m = m; s->total = total;
    return (Callable){ handler, s };
}
static void register_handler(Dispatcher *d, Callable c) { cvec_push(&d->hs, c); }

static int64_t handler(State *st, Dispatcher *d, int64_t k, int64_t v) {
    if (v >= 0) {
        st->counter += v * st->m;
        TOTAL_ADD(st->total);
        if (k % 100000 == 99999) {
            st->rc++;                                  /* used after a call that may release it */
            unregister_current(d);
            register_handler(d, make_handler(k % 17 + 1, st->total));
            int64_t c = st->counter;
            state_release(st);
            return c;
        }
    }
    return st->counter;
}

static inline void dispatch(Dispatcher *d, int64_t k) {
    d->current = k % (int64_t)d->hs.len;
    size_t i = (size_t)d->current;
    e_bounds(i, d->hs.len);
    Callable c = d->hs.ptr[i];                          /* copied out; not used after the call */
    c.fn(c.st, d, k, k % 1000);
}

int main(void) {
    h_start();
#ifdef BOXED
    Box64 *total = malloc(sizeof(Box64)); total->rc = 1; total->value = 0;
#else
    int64_t total_local = 0; int64_t *total = &total_local;
#endif
    Dispatcher *d = malloc(sizeof(Dispatcher));
    d->rc = 1; d->type = 1; d->hs = (CVec){0}; d->current = 0;
    for (int64_t i = 0; i < H; i++) register_handler(d, make_handler(i % 13 + 1, total));
    for (int64_t k = 0; k < E; k++) dispatch(d, k);
    int64_t s = 0;
    for (size_t i = 0; i < d->hs.len; i++) { Callable c = d->hs.ptr[i]; s += c.fn(c.st, d, 0, -1); }
#ifdef BOXED
    h_report("w4-ember-point-boxed", (uint64_t)(total->value + s));
#else
    h_report("w4-ember-point", (uint64_t)(total_local + s));
#endif
    return 0;
}
