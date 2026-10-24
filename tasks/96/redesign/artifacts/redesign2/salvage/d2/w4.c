/* SALVAGED "checked pointers" (phase3-design-D2.md), W4: callbacks with captured mutable state.
   Built from target/redesign/bench/D2/w4.c (closure state in its type's store with {generation, pins},
   an owned callable {function, reference to state}, `total` in a cell owned by main (P4), the state
   live on entry (P1), O5's deferred destroy pinned on the rare path only). Fixes:
   - FIX 1 (owned fields inline, the breaker's +26%): `handlers: list[own fn(Event)] = []` is a field
     initialised with a new object and never re-pointed, so the list lives inside the dispatcher's
     slot: the dispatcher reads its length and buffer from its own line, with no extra load through a
     separate list object. (The model as written did this without the design saying so; the rule is
     now stated: such a field is stored inline, and a reference to it is a reference to its owner
     plus the field's offset.)
   - FIX 2 (the reference): one 64-bit word {48-bit address, 16-bit generation}; following it is one
     AND, not base + index * size.
   Left out, as in every model: [TYP-8] overflow checks. */
#include "../sh.h"
#include <string.h>

#define H 10000
#define E 10000000LL
#define DOOMED 0x80000000u
static void d2_panic(const char *m) { fprintf(stderr, "panic: %s\n", m); fflush(stderr); ExitProcess(101); }

typedef uint64_t GRef;
#define GADDR(r) ((r) & 0x0000FFFFFFFFFFFFull)
#define GGEN(r) ((uint32_t)((r) >> 48))

typedef struct { uint32_t gen, pins; int64_t counter, m; GRef d, total; } HandlerState; /* 40 bytes */
typedef struct { void (*fn)(GRef, int64_t, int64_t); GRef state; } Callable;           /* 16 bytes */
typedef struct { Callable *ptr; int64_t len, cap; } CallList;
typedef struct { uint32_t gen, pad; CallList handlers; } DispatcherSlot;              /* list inline */
typedef struct { uint32_t gen, pad; int64_t value; } CellSlot;
#define HS(r) ((HandlerState *)GADDR(r))
#define DS(r) ((DispatcherSlot *)GADDR(r))
#define CS(r) ((CellSlot *)GADDR(r))

/* the handler-state store: 40-byte slots that never move (one chunk is enough here), LIFO free list */
static HandlerState *hstore; static uint32_t hlen = 1, hfree = UINT32_MAX;
static inline GRef href(HandlerState *h) { return (GRef)(uintptr_t)h | ((GRef)(h->gen & 0xFFFF) << 48); }
static GRef hstate_alloc(void) {
    uint32_t s = hfree;
    if (s != UINT32_MAX) hfree = (uint32_t)hstore[s].counter;
    else { s = hlen++; if (hlen > 1u << 20) d2_panic("store capacity exhausted"); }
    HandlerState *h = &hstore[s];
    h->gen += 1; h->pins = 0;
    return href(h);
}
static void hstate_free_now(HandlerState *h) {
    h->gen += 1; h->pins = 0;
    if ((h->gen & 0xFFFF) == 0xFFFE) return; /* retired */
    h->counter = (int64_t)hfree;
    hfree = (uint32_t)(h - hstore);
}
static void hstate_destroy(GRef r) {
    HandlerState *h = HS(r);
    if ((h->gen & 0xFFFF) != GGEN(r)) d2_panic("destroy: stale reference");
    if (h->pins & ~DOOMED) h->pins |= DOOMED; /* O5: its own code is running: destroy when it returns */
    else hstate_free_now(h);
}
static void calllist_push(CallList *l, Callable c) {
    if (l->len == l->cap) {
        int64_t nc = l->cap ? l->cap * 2 : 4;
        Callable *np = realloc(l->ptr, (size_t)nc * sizeof(Callable));
        if (!np) d2_panic("out of memory");
        l->ptr = np; l->cap = nc;
    }
    l->ptr[l->len++] = c;
}
static void handler_fn(GRef st, int64_t k, int64_t v);
static GRef make_handler(GRef d, GRef total, int64_t m) {
    GRef r = hstate_alloc();
    HandlerState *h = HS(r);
    h->counter = 0; h->m = m; h->d = d; h->total = total;
    return r;
}
static void dispatcher_register(DispatcherSlot *d, Callable c) { calllist_push(&d->handlers, c); }
static void dispatcher_unregister(DispatcherSlot *d, GRef st) {
    CallList *l = &d->handlers;
    for (int64_t i = 0; i < l->len; i++)
        if (l->ptr[i].state == st) {
            memmove(&l->ptr[i], &l->ptr[i + 1], (size_t)(l->len - i - 1) * sizeof(Callable));
            l->len--;
            hstate_destroy(st);
            return;
        }
    d2_panic("unregister: handler not registered");
}
static void handler_fn(GRef st, int64_t k, int64_t v) {
    HandlerState *h = HS(st);          /* P1: live on entry */
    h->counter += v * h->m;
    CS(h->total)->value += 1;           /* P4: main's cell */
    if (k % 100000 == 99999) {
        h->pins += 1;                   /* O5, on the rare path only */
        DispatcherSlot *d = DS(h->d);
        dispatcher_unregister(d, st);
        dispatcher_register(d, (Callable){handler_fn, make_handler(h->d, h->total, (k % 17) + 1)});
        h->pins -= 1;
        if (h->pins == DOOMED) hstate_free_now(h);
    }
}

int main(void) {
    h_start();
    hstore = VirtualAlloc(NULL, (size_t)40 << 20, MEM_RESERVE | MEM_COMMIT, PAGE_READWRITE);
    static DispatcherSlot dslot[2];
    static CellSlot cslot[2];
    if (!hstore) d2_panic("out of memory");
    hstore[0].gen = 2; dslot[0].gen = 2; cslot[0].gen = 2;
    cslot[1].gen = 1; dslot[1].gen = 1;
    GRef total = (GRef)(uintptr_t)&cslot[1] | (1ull << 48);
    GRef dref = (GRef)(uintptr_t)&dslot[1] | (1ull << 48);
    for (int64_t i = 0; i < H; i++)
        dispatcher_register(DS(dref), (Callable){handler_fn, make_handler(dref, total, (i % 13) + 1)});
    for (int64_t k = 0; k < E; k++) {
        DispatcherSlot *d = DS(dref);   /* owned by main (P4): no check */
        int64_t n = d->handlers.len;
        if (n == 0) d2_panic("integer division by zero");
        Callable c = d->handlers.ptr[k % n];
        c.fn(c.state, k, k % 1000);
    }
    uint64_t sum = (uint64_t)CS(total)->value;
    CallList *l = &DS(dref)->handlers;
    for (int64_t i = 0; i < l->len; i++) sum += (uint64_t)HS(l->ptr[i].state)->counter;
    h_report("d2-w4", sum);
    return 0;
}
