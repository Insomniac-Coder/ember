/* SALVAGED "today's rules, but never an error" (phase3-design-D3.md), W4: callbacks with captured state.
   What changed from target/redesign/bench/D3/w4.c, and why:
   - FIX (the pin moves to where the use is): D3 pinned d.handlers around EVERY call, because the
     closure's state lived inline in the list and the call may change the list. Here a closure whose
     state is written after it may be removed keeps that state in a box (one allocation per closure,
     as plain C's malloc'd environment), and the list holds {function, box} (16 bytes). The
     dispatcher copies the value out and calls it; it uses nothing of it afterwards, so it protects
     nothing. The handler's body uses its box after `d.unregister_current()` on its rare path, so the
     body pins its own box across that call (D2's O5): a removal that finds the box pinned marks it
     doomed and the box is freed at the unpin. The list itself is never pinned, so remove is a plain
     memmove (no tombstones, no overflow pages, no settle, no slow index path).
   - boxes come from the runtime's slab for their size class (32 bytes), as D3's objects do.
   - `total` is captured by reference as a plain pointer (D3's proof: every closure stored in `d`
     dies before `total`); `counter` and `m` are in the box.
   Left out, as in every model: [TYP-8] overflow checks. */
#include "../sh.h"
#include <string.h>

#define H 10000
#define E 10000000
static void panic(const char *m) { fprintf(stderr, "panic: %s\n", m); exit(101); }
#define CHECK(c, m) do { if (__builtin_expect(!(c), 0)) panic(m); } while (0)
static void *xrealloc(void *p, size_t n) { void *q = realloc(p, n); CHECK(q, "out of memory"); return q; }

typedef struct Dispatcher Dispatcher;
typedef struct { uint32_t pins, doomed; int64_t counter, m; int64_t *total; } HBox; /* 32 bytes */
typedef int64_t (*HFn)(HBox *, Dispatcher *, int64_t k, int64_t v);
typedef struct { HFn fn; HBox *box; } Handler;                                      /* 16 bytes */
typedef struct { Handler *data; int64_t len, cap; } HandlerList;
struct Dispatcher { HandlerList handlers; int64_t current; };

/* boxes come from the runtime's object slab for their size class (D3 section 1.2), bump then free list */
static char *slab_cur, *slab_end; static HBox *slab_free;
static HBox *box_alloc(void) {
    if (slab_free) { HBox *b = slab_free; slab_free = (HBox *)b->total; return b; }
    if (slab_cur == slab_end) { size_t sz = (size_t)4096 * sizeof(HBox); slab_cur = xrealloc(NULL, sz); slab_end = slab_cur + sz; }
    HBox *b = (HBox *)slab_cur; slab_cur += sizeof(HBox); return b;
}
static void box_free(HBox *b) { b->total = (int64_t *)slab_free; slab_free = b; }
static void box_destroy(HBox *b) { if (b->pins) b->doomed = 1; else box_free(b); }
static void d_register(Dispatcher *d, Handler h) {
    HandlerList *L = &d->handlers;
    if (L->len == L->cap) { int64_t c = L->cap ? L->cap * 2 : 4; L->data = xrealloc(L->data, (size_t)c * sizeof(Handler)); L->cap = c; }
    L->data[L->len++] = h;
}
static void d_unregister_current(Dispatcher *d) { /* the others keep their order */
    HandlerList *L = &d->handlers;
    int64_t i = d->current;
    CHECK(i >= 0 && i < L->len, "index out of bounds");
    HBox *b = L->data[i].box;
    memmove(&L->data[i], &L->data[i + 1], (size_t)(L->len - i - 1) * sizeof(Handler));
    L->len--;
    box_destroy(b);
}
static int64_t handler_fn(HBox *st, Dispatcher *d, int64_t k, int64_t v);
static Handler make_handler(int64_t m, int64_t *total) {
    HBox *b = box_alloc();
    *b = (HBox){0, 0, 0, m, total};
    return (Handler){handler_fn, b};
}
static int64_t handler_fn(HBox *st, Dispatcher *d, int64_t k, int64_t v) {
    if (v >= 0) {
        st->counter += v * st->m;
        *st->total += 1;
        if (k % 100000 == 99999) {
            st->pins++;                       /* st is used after a call that may destroy it */
            d_unregister_current(d);
            d_register(d, make_handler(k % 17 + 1, st->total));
            if (--st->pins == 0 && st->doomed) { int64_t c = st->counter; box_free(st); return c; }
        }
    }
    return st->counter;
}

int main(void) {
    h_start();
    int64_t total = 0;
    Dispatcher *d = calloc(1, sizeof *d);
    CHECK(d, "out of memory");
    for (int64_t i = 0; i < H; i++) d_register(d, make_handler(i % 13 + 1, &total));
    for (int64_t k = 0; k < E; k++) {
        HandlerList *L = &d->handlers;
        int64_t n = L->len;
        CHECK(n != 0, "integer division by zero");
        d->current = k % n;
        Handler h = L->data[d->current];       /* copied out; not used after the call */
        h.fn(h.box, d, k, k % 1000);
    }
    int64_t s = 0;
    for (int64_t i = 0; i < d->handlers.len; i++) { Handler h = d->handlers.data[i]; s += h.fn(h.box, d, 0, -1); }
    h_report("d3-w4", (uint64_t)(total + s));
    return 0;
}
