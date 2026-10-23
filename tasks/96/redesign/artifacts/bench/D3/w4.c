/* D3 model of W4 (callbacks with captured mutable state), as the D3 compiler would emit the program
   in tasks/96/redesign/phase3-design-D3.md (section 3, W4).

   What D3 puts here, and nothing else:
   - A handler is an owned callable value stored inline in the list: a function pointer plus its
     captured state, three words inline ([CLO-10]): its own `counter` (moved in: make_handler never
     uses it again), `m` (copied), and `total` captured BY REFERENCE as a plain pointer. The static
     proof: `total` is a local of main, the dispatcher `d` is a later local of main, so every closure
     stored in d.handlers dies before `total` (the region is carried by the list; no box, no count).
   - `d.handlers` is a List field of an object, and dispatch calls a closure stored in it while that
     closure may change it (it unregisters itself and registers another): the call PINS the list.
     The pin is a run-time counter, because the change happens inside methods also called unpinned.
     While pinned, `remove` leaves a tombstone (the running closure's state stays where it is) and
     `append` never moves an element (in place while there is capacity, else an overflow page). When
     the call returns and the pin count reaches 0, the list SETTLES (removals applied in order,
     overflow merged).
   - Indexing compares the index with `flen`, the length usable for direct indexing (the length
     when nothing is pending, else 0): one compare, the same as a bounds check; pending changes take
     the slow path. The handler's `v >= 0` test is the user's (it lets the final loop read counters).
   Left out on purpose, identical in every memory model: integer overflow checks ([TYP-8]).
   The pin record's side fields are inline here; the design keeps them in a side record reached only
   on the slow paths. */
#include "../harness.h"
#include <string.h>

#define H 10000
#define E 10000000

static void panic(const char *m) { fprintf(stderr, "panic: %s\n", m); exit(101); }
#define CHECK(c, m) do { if (__builtin_expect(!(c), 0)) panic(m); } while (0)
static void *xrealloc(void *p, size_t n) { void *q = realloc(p, n); CHECK(q, "out of memory"); return q; }

typedef struct Dispatcher Dispatcher;
typedef struct { int64_t counter, m; int64_t *total; } HState;             /* captured state */
typedef int64_t (*HFn)(HState *, Dispatcher *, int64_t k, int64_t v);
typedef struct { HFn fn; HState st; } Handler;                              /* 32 bytes; fn == NULL: tombstone */

#define OVP 1024
typedef struct {               /* a pinnable List field */
    Handler *data; int64_t len, cap;
    int64_t count;             /* the logical length: what len(xs) returns */
    int64_t flen;              /* len when nothing is pending, else 0 */
    uint32_t pins;
    int64_t ntombs;            /* removals waiting for settle */
    Handler **ovp; int64_t ov_len, ov_pcap;
} HandlerList;

struct Dispatcher { HandlerList handlers; int64_t current; };

static Handler *hl_slot(HandlerList *L, int64_t phys) {   /* physical slot, overflow included */
    if (phys < L->len) return &L->data[phys];
    int64_t k = phys - L->len;
    return &L->ovp[k / OVP][k % OVP];
}
static inline int64_t hl_len(HandlerList *L) { return L->count; }
static Handler *hl_at_slow(HandlerList *L, int64_t idx) {  /* skip tombstones */
    CHECK(idx >= 0 && idx < hl_len(L), "index out of bounds");
    for (int64_t phys = 0, seen = 0;; phys++) {
        Handler *h = hl_slot(L, phys);
        if (h->fn) { if (seen == idx) return h; seen++; }
    }
}
static inline Handler *hl_at(HandlerList *L, int64_t idx) {
    if (__builtin_expect((uint64_t)idx < (uint64_t)L->flen, 1)) return &L->data[idx];
    return hl_at_slow(L, idx);
}
static void hl_push(HandlerList *L, Handler h) {
    if (L->len < L->cap && L->ov_len == 0) {
        L->data[L->len++] = h;
    } else if (L->pins == 0) {            /* not pinned: grow by moving */
        int64_t c = L->cap ? L->cap * 2 : 4;
        L->data = xrealloc(L->data, (size_t)c * sizeof(Handler));
        L->cap = c;
        L->data[L->len++] = h;
    } else {                              /* pinned and full: an overflow page */
        int64_t k = L->ov_len;
        if (k % OVP == 0) {
            if (k / OVP == L->ov_pcap) {
                L->ov_pcap = L->ov_pcap ? L->ov_pcap * 2 : 4;
                L->ovp = xrealloc(L->ovp, (size_t)L->ov_pcap * sizeof(Handler *));
            }
            L->ovp[k / OVP] = xrealloc(NULL, OVP * sizeof(Handler));
        }
        L->ovp[k / OVP][k % OVP] = h;
        L->ov_len++;
    }
    L->count++;
    L->flen = (L->ntombs == 0 && L->ov_len == 0) ? L->len : 0;
}
static void hl_remove_at(HandlerList *L, int64_t idx) {   /* the others keep their order */
    Handler *h = hl_at(L, idx);
    if (L->pins) {                        /* pinned: a tombstone; the state is not moved or freed */
        h->fn = NULL;
        L->ntombs++;
        L->count--;
        L->flen = 0;
    } else {
        int64_t i = h - L->data;          /* unpinned: no overflow, no tombstones */
        memmove(&L->data[i], &L->data[i + 1], (size_t)(L->len - i - 1) * sizeof(Handler));
        L->len--;
        L->count--;
        L->flen = L->len;
    }
}
static void hl_settle(HandlerList *L) {
    if (L->ov_len) {
        int64_t need = L->len + L->ov_len, c = L->cap ? L->cap : 4;
        while (c < need) c *= 2;
        L->data = xrealloc(L->data, (size_t)c * sizeof(Handler));
        L->cap = c;
        for (int64_t k = 0; k < L->ov_len; k++) L->data[L->len + k] = L->ovp[k / OVP][k % OVP];
        for (int64_t pg = 0; pg * OVP < L->ov_len; pg++) free(L->ovp[pg]);
        L->len = need; L->ov_len = 0;
    }
    if (L->ntombs) {                      /* close the gaps, keeping the order */
        int64_t j = 0;
        for (int64_t i = 0; i < L->len; i++)
            if (L->data[i].fn) { if (i != j) L->data[j] = L->data[i]; j++; }
        L->len = j; L->ntombs = 0;
    }
    L->flen = L->len;
}
static inline void hl_unpin(HandlerList *L) {
    if (--L->pins == 0 && (L->ntombs | L->ov_len)) hl_settle(L);
}

/* Dispatcher methods */
static void d_register(Dispatcher *d, Handler h) { hl_push(&d->handlers, h); }
static void d_unregister_current(Dispatcher *d) { hl_remove_at(&d->handlers, d->current); }
static void d_dispatch(Dispatcher *d, int64_t k) {
    HandlerList *L = &d->handlers;
    int64_t n = hl_len(L);
    d->current = k % n;
    Handler *h = hl_at(L, d->current);
    L->pins++;                            /* the closure may change d.handlers */
    h->fn(&h->st, d, k, k % 1000);
    hl_unpin(L);
}

/* make_handler(m): a local function of main capturing `total` by reference */
static int64_t handler_fn(HState *st, Dispatcher *d, int64_t k, int64_t v);
static inline Handler make_handler(int64_t m, int64_t *total) {
    return (Handler){ handler_fn, { 0, m, total } };
}
static int64_t handler_fn(HState *st, Dispatcher *d, int64_t k, int64_t v) {
    if (v >= 0) {
        st->counter += v * st->m;
        *st->total += 1;
        if (k % 100000 == 99999) {
            d_unregister_current(d);      /* pinned: a tombstone; st stays valid */
            d_register(d, make_handler(k % 17 + 1, st->total));
        }
    }
    return st->counter;
}

int main(void) {
    h_start();
    int64_t total = 0;
    Dispatcher *d = calloc(1, sizeof *d); /* a local object of main, declared after `total` */
    CHECK(d, "out of memory");
    for (int64_t i = 0; i < H; i++) d_register(d, make_handler(i % 13 + 1, &total));
    for (int64_t k = 0; k < E; k++) d_dispatch(d, k);

    /* for h in d.handlers: s += h(d, 0, -1)   (the loop pins the list: its body calls closures) */
    HandlerList *L = &d->handlers;
    int64_t s = 0;
    L->pins++;
    int64_t n0 = L->len + L->ov_len;
    for (int64_t i = 0; i < n0; i++) {
        Handler *h = hl_slot(L, i);
        if (!h->fn) continue;
        s += h->fn(&h->st, d, 0, -1);
    }
    hl_unpin(L);
    h_report("D3-w4", (uint64_t)(total + s));
    return 0;
}
