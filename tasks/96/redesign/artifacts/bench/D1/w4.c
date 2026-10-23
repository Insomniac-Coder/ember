/* D1 (values and projections), workload W4: callbacks with captured mutable state.
   Report: tasks/96/redesign/phase3-design-D1.md (sections 3, 4, 5).

   The user program: `total` is a local of main; `make_handler(m)` is a local function of main
   with a local `counter` and two closures, `on_event(d, k, v)` (writes counter and total) and
   `fn() => counter`; `class Dispatcher` holds `handlers: Array[Handler]` and calls
   `self.handlers[j].on_event(self, k, v)`.
   What D1 needs and is therefore here:
   - variables that an escaping closure captures live in one counted box per call frame (Python's
     cells, grouped per frame): main's box holds `total`; each make_handler call's box holds
     `counter`, `m` and the link to main's box (the scope chain, retained);
   - a closure value is (function pointer, box pointer); a Handler holds two;
   - calling a closure stored in an object's list copies the closure value out and retains its
     box for the call, because the call may remove that very handler (it does, every 100,000
     events); after the call the box is released; no address into the list is held;
   - `remove(i)` and `push` test uniqueness; remove checks the index, shifts the rest, bumps the
     shift version and releases the removed closures;
   - `d` is a class object (16-byte header); its fields carry no access words in D1;
   - `k % self.handlers.len()`: a zero length panics (division by zero); the index is then proven.
   Left out on purpose, the same in every memory model: the integer overflow checks of [TYP-8].
   No teardown at the end: memory-only values whose last use ends main are not freed. */
#include "../harness.h"
#include <string.h>

static _Noreturn void d1_panic(const char *msg) {
    fprintf(stderr, "panic: %s\n", msg);
    fflush(stderr);
    ExitProcess(101);
}
static void *xmalloc(size_t n) {
    void *p = malloc(n);
    if (!p) d1_panic("out of memory");
    return p;
}

/* ---- D1 runtime: counted buffers (Array) ---- */
typedef struct Buf { uint32_t count; uint32_t shifts; uint64_t len; uint64_t cap; } Buf; /* 24 bytes */
static Buf EMPTY = {0x80000000u, 0, 0, 0};
#define EL(b, T) ((T *)((Buf *)(b) + 1))
typedef void (*RetainFn)(void *elems, uint64_t n);

static Buf *buf_reserve_slow(Buf **slot, size_t esz, uint64_t need, RetainFn retain) {
    Buf *b = *slot;
    uint64_t cap = b->cap;
    if (need > cap) {
        uint64_t nc = cap < 4 ? 4 : cap * 2;
        while (nc < need) nc *= 2;
        cap = nc;
    }
    if (b->count == 1) {
        b = realloc(b, sizeof(Buf) + cap * esz);
        if (!b) d1_panic("out of memory");
        b->cap = cap;
    } else {
        Buf *nb = xmalloc(sizeof(Buf) + cap * esz);
        nb->count = 1; nb->shifts = 0; nb->len = b->len; nb->cap = cap;
        memcpy(nb + 1, b + 1, b->len * esz);
        if (retain) retain(nb + 1, b->len);
        if (b != &EMPTY) b->count--;
        b = nb;
    }
    *slot = b;
    return b;
}
static inline Buf *buf_mut(Buf **slot, size_t esz, RetainFn retain) {
    Buf *b = *slot;
    if (__builtin_expect(b->count != 1, 0)) b = buf_reserve_slow(slot, esz, b->len, retain);
    return b;
}
#define PUSH(slot, T, val, retain)                                                                   \
    do {                                                                                             \
        Buf *b_ = *(slot);                                                                           \
        if (__builtin_expect(b_->count != 1 || b_->len == b_->cap, 0))                               \
            b_ = buf_reserve_slow((slot), sizeof(T), b_->len + 1, (retain));                         \
        EL(b_, T)[b_->len++] = (val);                                                                \
    } while (0)

/* ---- the program ---- */
typedef struct Dispatcher Dispatcher;
typedef struct MainBox { uint32_t rc; int64_t total; } MainBox;                    /* main's captured frame */
typedef struct HBox { uint32_t rc; int64_t counter; int64_t m; MainBox *up; } HBox; /* one make_handler call's */
typedef void (*EventFn)(void *box, Dispatcher *d, int64_t k, int64_t v);
typedef int64_t (*CountFn)(void *box);
typedef struct Handler { EventFn on_event; void *ev_box; CountFn counter; void *cnt_box; } Handler; /* 32 bytes */
typedef struct TypeInfo { const char *name; } TypeInfo;
struct Dispatcher {
    uint32_t strong, weak;  /* header */
    const TypeInfo *type;   /* header */
    Buf *handlers;          /* Array[Handler] */
    int64_t current;
};
static const TypeInfo TI_DISPATCHER = {"Dispatcher"};

static void hbox_release(HBox *b) {
    if (--b->rc == 0) {
        MainBox *up = b->up;
        free(b);
        if (--up->rc == 0) free(up);
    }
}
static void retain_handlers(void *el, uint64_t n) { /* copying a Handler retains both closures' boxes */
    Handler *h = el;
    for (uint64_t i = 0; i < n; i++) { ((HBox *)h[i].ev_box)->rc++; ((HBox *)h[i].cnt_box)->rc++; }
}

static void dispatcher_register(Dispatcher *d, Handler h) { /* self.handlers.push(h): h moved in */
    PUSH(&d->handlers, Handler, h, retain_handlers);
}
static void dispatcher_unregister_current(Dispatcher *d) { /* self.handlers.remove(self.current) */
    Buf *b = buf_mut(&d->handlers, sizeof(Handler), retain_handlers);
    uint64_t j = (uint64_t)d->current;
    if (d->current < 0 || j >= b->len) d1_panic("index out of bounds");
    Handler h = EL(b, Handler)[j];
    memmove(&EL(b, Handler)[j], &EL(b, Handler)[j + 1], (b->len - j - 1) * sizeof(Handler));
    b->len--; b->shifts++;
    hbox_release(h.ev_box); /* the removed value is dropped: both closures */
    hbox_release(h.cnt_box);
}

static Handler make_handler(MainBox *mb, int64_t m);

/* fn on_event(d, k, v): counter += v * m; total += 1; if k % 100_000 == 99_999: ... */
static void on_event(void *boxp, Dispatcher *d, int64_t k, int64_t v) {
    HBox *b = boxp;
    b->counter += v * b->m;
    b->up->total += 1;
    if (k % 100000 == 99999) {
        dispatcher_unregister_current(d);                        /* removes this very handler */
        dispatcher_register(d, make_handler(b->up, k % 17 + 1)); /* b is still alive: retained by the caller */
    }
}
static int64_t counter_of(void *boxp) { return ((HBox *)boxp)->counter; } /* fn() => counter */

static Handler make_handler(MainBox *mb, int64_t m) {
    HBox *b = xmalloc(sizeof *b);
    b->rc = 2; /* two closures share the frame's box */
    b->counter = 0; b->m = m;
    b->up = mb; mb->rc++;
    return (Handler){on_event, b, counter_of, b};
}

int main(void) {
    h_start();
    enum { H = 10000 };
    const int64_t E = 10000000;
    MainBox *mb = xmalloc(sizeof *mb); /* total = 0, boxed: escaping closures write it */
    mb->rc = 1; mb->total = 0;

    Dispatcher *d = xmalloc(sizeof *d);
    d->strong = 1; d->weak = 1; d->type = &TI_DISPATCHER; d->handlers = &EMPTY; d->current = 0;

    for (int64_t i = 0; i < H; i++) dispatcher_register(d, make_handler(mb, i % 13 + 1));

    for (int64_t k = 0; k < E; k++) {
        /* d.dispatch(k): self.current = k % self.handlers.len(); self.handlers[self.current].on_event(self, k, k % 1000) */
        const Buf *hb = d->handlers;
        uint64_t n = hb->len;
        if (n == 0) d1_panic("division by zero");
        uint64_t j = (uint64_t)k % n;
        d->current = (int64_t)j;
        Handler h = EL(hb, Handler)[j];   /* the closure value, copied out */
        HBox *b = h.ev_box;
        b->rc++;                          /* retained for the call: the call may remove it */
        h.on_event(b, d, k, k % 1000);
        hbox_release(b);
    }

    int64_t checksum = mb->total;
    for (uint64_t i = 0; i < d->handlers->len; i++) { /* for h in d.handlers: checksum += h.counter() */
        /* the loop holds a path: the list is fetched again after each opaque call */
        Handler h = EL(d->handlers, Handler)[i];
        HBox *b = h.cnt_box;
        b->rc++;
        checksum += h.counter(b);
        hbox_release(b);
    }
    h_report("D1-w4", (uint64_t)checksum);
    return 0;
}
