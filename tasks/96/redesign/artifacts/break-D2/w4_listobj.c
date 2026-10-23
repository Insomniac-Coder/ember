/* D2 (generational references + stable stores), workload W4: callbacks with captured mutable state.
   What the D2 compiler would emit for tasks/96/redesign/phase3-design-D2.md section 3, W4:
   - a stateful closure stored in a list is an object: its captured state lives in a slot of the
     closure type's store (never moves), {u32 generation, u32 pins, counter, m, d, total};
   - an owned callable value is 16 bytes, {function pointer, reference to its state};
   - `d.handlers: list[own fn(Event)]` owns the closures; removing one destroys its state;
   - `total` is written by escaping closures, so it lives in a cell object owned by main's frame;
     main never reassigns or deletes it, so references to it need no check (proof P4); the same for d;
   - the closure's own state is live on entry: the caller read the callable from its owning list
     (proof P1);
   - an object is never destroyed while its own code runs: before the first call that may destroy
     the running closure (the unregister), its state is pinned; the destruction completes when the
     call returns. The pin is on the rare path only;
   - `k % len(d.handlers)` keeps its division-by-zero check; the index is in range by construction.
   Not modelled (outside the memory model, same for every design): integer overflow checks. */
#include "../bench/harness.h"
#include <string.h>

typedef uint32_t u32;
typedef uint64_t u64;
typedef int64_t i64;

#define H 10000
#define E 10000000LL
#define NO_SLOT UINT32_MAX
#define DOOMED 0x80000000u

static void d2_panic(const char *m) {
    fprintf(stderr, "panic: %s\n", m);
    fflush(stderr);
    ExitProcess(101);
}

typedef struct { char *base; size_t reserved, committed; } Range;

static void range_init(Range *r, size_t reserve) {
    r->base = (char *)VirtualAlloc(NULL, reserve, MEM_RESERVE, PAGE_NOACCESS);
    if (!r->base) d2_panic("address space reservation failed");
    r->reserved = reserve;
    r->committed = 0;
}

static void range_ensure(Range *r, size_t bytes) {
    if (bytes <= r->committed) return;
    size_t chunk = (size_t)1 << 20;
    size_t want = (bytes + chunk - 1) & ~(chunk - 1);
    if (want > r->reserved) d2_panic("store capacity exhausted");
    if (!VirtualAlloc(r->base + r->committed, want - r->committed, MEM_COMMIT, PAGE_READWRITE))
        d2_panic("out of memory");
    r->committed = want;
}

typedef struct { u32 idx, gen; } Ref;

/* the state of `fn handler(ev)` made by make_handler */
typedef struct {
    u32 gen;
    u32 pins;   /* running calls that may destroy it; DOOMED = destroy when the last returns */
    i64 counter;
    i64 m;
    Ref d;      /* captured reference to the dispatcher */
    Ref total;  /* captured reference to main's `total` cell */
} HandlerState; /* 40 bytes */

typedef struct { void (*fn)(Ref, i64, i64); Ref state; } Callable; /* owned callable value */
typedef struct { Callable *ptr; i64 len, cap; } CallList;
/* PROBE: `handlers: list[own fn(Event)] = []` as D2 section 1.1 says: a list object in the list
   store; the dispatcher holds an 8-byte owning reference to it. */
typedef struct { u32 gen; u32 pad; CallList l; } CallListSlot;
static CallListSlot *cl_base;
static inline CallList *hl(Ref r) { return &cl_base[r.idx].l; } /* P1: read from its owning field */
typedef struct { u32 gen; u32 pad; Ref handlers; } DispatcherSlot;
typedef struct { u32 gen; u32 pad; i64 value; } CellSlot;

/* one store per class: slot 0 reserved */
static Range hstore_r, dstore_r, cstore_r;
static u32 hstore_len = 1, hstore_free = NO_SLOT;

static inline HandlerState *hstate(u32 i) { return (HandlerState *)hstore_r.base + i; }
static inline DispatcherSlot *disp(u32 i) { return (DispatcherSlot *)dstore_r.base + i; }
static inline CellSlot *cell(u32 i) { return (CellSlot *)cstore_r.base + i; }

static Ref hstate_alloc(void) {
    u32 s = hstore_free;
    if (s != NO_SLOT) {
        hstore_free = (u32)hstate(s)->counter; /* free-list link kept in the dead slot */
    } else {
        s = hstore_len++;
        range_ensure(&hstore_r, (size_t)hstore_len * sizeof(HandlerState));
    }
    HandlerState *h = hstate(s);
    h->gen += 1;
    h->pins = 0;
    return (Ref){s, h->gen};
}

static void hstate_free_now(u32 s) {
    HandlerState *h = hstate(s);
    h->gen += 1; /* even: stale */
    h->pins = 0;
    if (h->gen == UINT32_MAX - 1) return; /* retired */
    h->counter = (i64)hstore_free;
    hstore_free = s;
}

/* destroy: deferred while the closure's own code is running */
static void hstate_destroy(u32 s) {
    HandlerState *h = hstate(s);
    if (h->pins & ~DOOMED) h->pins |= DOOMED;
    else hstate_free_now(s);
}

static void calllist_push(CallList *l, Callable c) {
    if (l->len == l->cap) {
        i64 nc = l->cap ? l->cap * 2 : 4;
        Callable *np = (Callable *)realloc(l->ptr, (size_t)nc * sizeof(Callable));
        if (!np) d2_panic("out of memory");
        l->ptr = np;
        l->cap = nc;
    }
    l->ptr[l->len++] = c;
}

static void handler_fn(Ref st, i64 k, i64 v);

static Ref make_handler(Ref d, Ref total, i64 m) {
    Ref r = hstate_alloc();
    HandlerState *h = hstate(r.idx);
    h->counter = 0;
    h->m = m;
    h->d = d;
    h->total = total;
    return r;
}

/* d.register(h) */
static void dispatcher_register(DispatcherSlot *d, Callable c) { calllist_push(hl(d->handlers), c); }

/* d.unregister(h): remove the element that is this closure, keep the others' order, destroy it */
static void dispatcher_unregister(DispatcherSlot *d, Ref st) {
    CallList *l = hl(d->handlers);
    for (i64 i = 0; i < l->len; i++) {
        if (l->ptr[i].state.idx == st.idx && l->ptr[i].state.gen == st.gen) {
            memmove(&l->ptr[i], &l->ptr[i + 1], (size_t)(l->len - i - 1) * sizeof(Callable));
            l->len--;
            hstate_destroy(st.idx);
            return;
        }
    }
    d2_panic("unregister: handler not registered");
}

static void handler_fn(Ref st, i64 k, i64 v) {
    HandlerState *h = hstate(st.idx); /* live on entry (P1) */
    h->counter += v * h->m;
    cell(h->total.idx)->value += 1;   /* P4: main's cell, no check */
    if (k % 100000 == 99999) {
        h->pins += 1;                 /* pinned before the call that may destroy it */
        DispatcherSlot *d = disp(h->d.idx); /* P4 */
        dispatcher_unregister(d, st);
        Ref nst = make_handler(h->d, h->total, (k % 17) + 1); /* h is pinned: its captures are readable */
        dispatcher_register(d, (Callable){handler_fn, nst});
        h->pins -= 1;
        if (h->pins == DOOMED) hstate_free_now(st.idx);
    }
}

int main(void) {
    h_start();
    range_init(&hstore_r, (size_t)64 << 20);
    range_init(&dstore_r, (size_t)1 << 20);
    range_init(&cstore_r, (size_t)1 << 20);
    range_ensure(&hstore_r, 2 * sizeof(HandlerState));
    hstate(0)->gen = 2;
    range_ensure(&dstore_r, 2 * sizeof(DispatcherSlot));
    disp(0)->gen = 2;
    range_ensure(&cstore_r, 2 * sizeof(CellSlot));
    cell(0)->gen = 2;

    /* total = 0 (a cell owned by main), d = Dispatcher() (owned by main) */
    Ref total = {1, 1};
    cell(1)->gen = 1;
    cell(1)->value = 0;
    Ref dref = {1, 1};
    disp(1)->gen = 1;
    static Range clr; range_init(&clr, (size_t)1 << 20); range_ensure(&clr, 2 * sizeof(CallListSlot));
    cl_base = (CallListSlot *)clr.base; cl_base[0].gen = 2; cl_base[1].gen = 1; cl_base[1].l = (CallList){0};
    disp(1)->handlers = (Ref){1, 1};

    for (i64 i = 0; i < H; i++)
        dispatcher_register(disp(dref.idx), (Callable){handler_fn, make_handler(dref, total, (i % 13) + 1)});

    for (i64 k = 0; k < E; k++) {
        DispatcherSlot *d = disp(dref.idx); /* owned by main (P1): no check */
        CallList *hs = hl(d->handlers);
        i64 n = hs->len;
        if (n == 0) d2_panic("integer division by zero");
        Callable c = hs->ptr[k % n]; /* in range by construction */
        c.fn(c.state, k, k % 1000);
    }

    u64 sum = (u64)cell(total.idx)->value;
    CallList *l = hl(disp(dref.idx)->handlers);
    for (i64 i = 0; i < l->len; i++) sum += (u64)hstate(l->ptr[i].state.idx)->counter;

    h_report("D2-w4-listobj", sum);
    return 0;
}
