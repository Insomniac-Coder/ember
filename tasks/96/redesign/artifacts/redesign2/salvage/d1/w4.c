/* SALVAGED "copy instead of share" (phase3-design-D1.md), W4: callbacks with captured mutable state.
   Built from target/redesign/bench/D1/w4.c (one counted box per closure frame, a Handler of two
   closures, counted buffers, remove shifting and releasing, the division-by-zero check). Fix:
   - FIX (who keeps the box alive): D1-7 made the CALLER retain a stored closure's box around every
     call (a count pair on each of 10,000,000 events) because the call may remove that closure. The
     fixed rule puts the protection where the use is: a function that uses a counted handle (its own
     box included) after a call that may release it retains the handle across that call. The
     handler's body uses its box after `d.unregister_current()` on its rare path, so it retains
     itself there (once per 100,000 events); the dispatcher, which does not use the box after the
     call, retains nothing.
   FIX 2 (allocation): closure boxes come from a 32-byte size-class slab instead of malloc. Built with
   -DMALLOCBOX the boxes are malloc'd again (FIX 1 alone); plain/w4_env32.c shows that one more 4-byte
   word in a malloc'd environment slows plain C itself.
   Built with -DONECLOSURE (attribution): one closure per Handler (16 bytes, the counter read from the box).
   Built with -DARRAY3: an Array value keeps its length beside its buffer pointer ({buffer, len}; the
   count stays in the buffer), so the dispatcher reads both from the object's own line instead of
   loading the length through the pointer: an experiment on D1's one-word Array value.
   Left out, as in every model: [TYP-8] overflow checks; teardown. */
#include "../sh.h"
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
#ifdef ONECLOSURE /* attribution build: the program reads the counter from the box, one closure per Handler */
typedef struct Handler { EventFn on_event; void *ev_box; } Handler; /* 16 bytes */
#else
typedef struct Handler { EventFn on_event; void *ev_box; CountFn counter; void *cnt_box; } Handler; /* 32 bytes */
#endif
typedef struct TypeInfo { const char *name; } TypeInfo;
struct Dispatcher {
    uint32_t strong, weak;  /* header */
    const TypeInfo *type;   /* header */
    Buf *handlers;          /* Array[Handler] */
#ifdef ARRAY3
    uint64_t hlen;          /* ARRAY3: the Array value is {buffer, len}, so len sits beside the pointer */
#endif
    int64_t current;
};
static const TypeInfo TI_DISPATCHER = {"Dispatcher"};

#ifndef MALLOCBOX /* FIX 2: closure boxes come from a 32-byte size-class slab (no heap header, 32-byte stride) */
static char *bslab_cur, *bslab_end; static HBox *bslab_free;
static HBox *hbox_alloc(void) {
    if (bslab_free) { HBox *b = bslab_free; bslab_free = (HBox *)b->up; return b; }
    if (bslab_cur == bslab_end) { size_t sz = (size_t)4096 * sizeof(HBox); bslab_cur = xmalloc(sz); bslab_end = bslab_cur + sz; }
    HBox *b = (HBox *)bslab_cur; bslab_cur += sizeof(HBox); return b;
}
static void hbox_free(HBox *b) { b->up = (MainBox *)bslab_free; bslab_free = b; }
#else
static HBox *hbox_alloc(void) { return xmalloc(sizeof(HBox)); }
static void hbox_free(HBox *b) { free(b); }
#endif
static void hbox_release(HBox *b) {
    if (--b->rc == 0) {
        MainBox *up = b->up;
        hbox_free(b);
        if (--up->rc == 0) free(up);
    }
}
static void retain_handlers(void *el, uint64_t n) { /* copying a Handler retains both closures' boxes */
    Handler *h = el;
#ifdef ONECLOSURE
    for (uint64_t i = 0; i < n; i++) ((HBox *)h[i].ev_box)->rc++;
#else
    for (uint64_t i = 0; i < n; i++) { ((HBox *)h[i].ev_box)->rc++; ((HBox *)h[i].cnt_box)->rc++; }
#endif
}

static void dispatcher_register(Dispatcher *d, Handler h) { /* self.handlers.push(h): h moved in */
    PUSH(&d->handlers, Handler, h, retain_handlers);
#ifdef ARRAY3
    d->hlen = d->handlers->len;
#endif
}
static void dispatcher_unregister_current(Dispatcher *d) { /* self.handlers.remove(self.current) */
    Buf *b = buf_mut(&d->handlers, sizeof(Handler), retain_handlers);
    uint64_t j = (uint64_t)d->current;
    if (d->current < 0 || j >= b->len) d1_panic("index out of bounds");
    Handler h = EL(b, Handler)[j];
    memmove(&EL(b, Handler)[j], &EL(b, Handler)[j + 1], (b->len - j - 1) * sizeof(Handler));
    b->len--; b->shifts++;
#ifdef ARRAY3
    d->hlen = b->len;
#endif
    hbox_release(h.ev_box); /* the removed value is dropped: both closures */
#ifndef ONECLOSURE
    hbox_release(h.cnt_box);
#endif
}

static Handler make_handler(MainBox *mb, int64_t m);

/* fn on_event(d, k, v): counter += v * m; total += 1; if k % 100_000 == 99_999: ... */
static void on_event(void *boxp, Dispatcher *d, int64_t k, int64_t v) {
    HBox *b = boxp;
    b->counter += v * b->m;
    b->up->total += 1;
    if (k % 100000 == 99999) {
        b->rc++;                                                 /* b is used after a call that may release it */
        dispatcher_unregister_current(d);                        /* removes this very handler */
        dispatcher_register(d, make_handler(b->up, k % 17 + 1));
        hbox_release(b);
    }
}
static int64_t counter_of(void *boxp) { return ((HBox *)boxp)->counter; } /* fn() => counter */

static Handler make_handler(MainBox *mb, int64_t m) {
    HBox *b = hbox_alloc();
    b->rc = 2; /* two closures share the frame's box */
    b->counter = 0; b->m = m;
    b->up = mb; mb->rc++;
#ifdef ONECLOSURE
    b->rc = 1;
    return (Handler){on_event, b};
#else
    return (Handler){on_event, b, counter_of, b};
#endif
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
#ifdef ARRAY3
        uint64_t n = d->hlen;
#else
        uint64_t n = hb->len;
#endif
        if (n == 0) d1_panic("division by zero");
        uint64_t j = (uint64_t)k % n;
        d->current = (int64_t)j;
        Handler h = EL(hb, Handler)[j];   /* the closure value, copied out */
        h.on_event(h.ev_box, d, k, k % 1000); /* not used after the call: no retain (callee protects itself) */
    }

    int64_t checksum = mb->total;
    for (uint64_t i = 0; i < d->handlers->len; i++) { /* for h in d.handlers: checksum += h.counter() */
        /* the loop holds a path: the list is fetched again after each opaque call */
        Handler h = EL(d->handlers, Handler)[i];
#ifdef ONECLOSURE
        checksum += ((HBox *)h.ev_box)->counter;
#else
        HBox *b = h.cnt_box;
        b->rc++;
        checksum += h.counter(b);
        hbox_release(b);
#endif
    }
    #ifdef ARRAY3
    h_report("d1-w4-array3", (uint64_t)checksum);
#else
    h_report("d1-w4", (uint64_t)checksum);
#endif
    return 0;
}
