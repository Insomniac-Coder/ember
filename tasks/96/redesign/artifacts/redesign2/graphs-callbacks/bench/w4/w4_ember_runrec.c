/* W4, Ember: the same program and the same closure rules as w4_ember.c, with a second way to make the
   call of a stored closure safe when the call may change the list that holds it ("running record"):

   - A closure's body already copies its inline state into its own frame on entry and writes it back
     on exit (today's emitted trampoline does this: probes/a32_emit_uaf/c/release/c/main.c, lines
     126-132). Today the write-back goes to the slot's old address, which is the use after free this
     report found. Here the write-back goes through a running record: {where the slot is now}.
   - The caller pushes the record on a per-thread list for the call, and puts a stub in the slot's
     function word (a re-entry of the running closure reaches the stub and panics).
   - An Array operation that moves or removes elements of an Array whose element type holds a callable
     (only those: the element type is known when the Array is compiled for it) first looks at the
     per-thread list (one load and a branch when it is empty). If a running slot moves, its record is
     updated; if it is removed, its record is cleared, and the closure drops its state on exit.
   Built with -DBOXED, `total` is in a counted box, as in w4_ember.c. */
#include "../ember_model.h"
#define H 10000
#define E 10000000

typedef struct Dispatcher Dispatcher;
typedef struct Callable Callable;
typedef struct RunRec { Callable *slot; struct RunRec *prev; } RunRec;
typedef int64_t (*HandlerFn)(RunRec *rec, Dispatcher *d, int64_t k, int64_t v);
struct Callable { HandlerFn fn; uintptr_t w[3]; };                /* [CLO-10]: 32 bytes */
typedef struct { Callable *ptr; size_t len, cap; } CVec;
struct Dispatcher { uint32_t rc, type; CVec hs; int64_t current; };

static RunRec *g_running;     /* per thread (_Thread_local in a threaded runtime) */

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

static int64_t running_stub(RunRec *rec, Dispatcher *d, int64_t k, int64_t v) {
    (void)rec; (void)d; (void)k; (void)v;
    e_panic("this handler is already running (re-entered from its own call)");
    return 0;
}

/* the fix-up an Array of callables runs before it moves elements [from, to) by `delta` slots or
   (delta == 0) removes the slot `from` */
static void running_fixup(Callable *from, Callable *to, ptrdiff_t delta, int removing) {
    for (RunRec *r = g_running; r; r = r->prev) {
        if (!r->slot) continue;
        if (removing && r->slot == from) r->slot = NULL;
        else if (r->slot >= from && r->slot < to) r->slot += delta;
    }
}

static void cvec_push(CVec *v, Callable c) {
    if (v->len == v->cap) {
        size_t cap = v->cap < 2 ? 4 : v->cap * 2;
        Callable *old = v->ptr;
        Callable *nw = malloc(cap * sizeof(Callable));
        if (v->len) memcpy(nw, old, v->len * sizeof(Callable));
        if (g_running) running_fixup(old, old + v->len, nw - old, 0);
        free(old);
        v->ptr = nw; v->cap = cap;
    }
    v->ptr[v->len++] = c;
}

static void drop_state(uintptr_t *w);

static void unregister_current(Dispatcher *d) {
    size_t i = (size_t)d->current;
    e_bounds(i, d->hs.len);
    Callable gone = d->hs.ptr[i];
    if (g_running) {
        running_fixup(&d->hs.ptr[i], &d->hs.ptr[i] + 1, 0, 1);
        running_fixup(&d->hs.ptr[i + 1], d->hs.ptr + d->hs.len, -1, 0);
    }
    memmove(&d->hs.ptr[i], &d->hs.ptr[i + 1], sizeof(Callable) * (d->hs.len - i - 1));
    d->hs.len--;
    if (gone.fn != running_stub) drop_state(gone.w);
}

static int64_t handler(RunRec *rec, Dispatcher *d, int64_t k, int64_t v);
static Callable make_handler(int64_t m, TOTAL_T total) {
    TOTAL_RETAIN(total);
    Callable c = { handler, { 0, (uintptr_t)m, (uintptr_t)total } };
    return c;
}
static void drop_state(uintptr_t *w) { TOTAL_RELEASE((TOTAL_T)w[2]); }
static void register_handler(Dispatcher *d, Callable c) { cvec_push(&d->hs, c); }

/* the closure body, with its state copied in on entry and written back through the record on exit */
static int64_t handler(RunRec *rec, Dispatcher *d, int64_t k, int64_t v) {
    uintptr_t *in = rec->slot->w;
    int64_t counter = (int64_t)in[0], m = (int64_t)in[1];
    TOTAL_T total = (TOTAL_T)in[2];
    if (v >= 0) {
        counter += v * m;
        TOTAL_ADD(total);
        if (k % 100000 == 99999) {
            unregister_current(d);
            register_handler(d, make_handler(k % 17 + 1, total));
        }
    }
    if (rec->slot) { rec->slot->w[0] = (uintptr_t)counter; rec->slot->w[1] = (uintptr_t)m; rec->slot->w[2] = (uintptr_t)total; }
    else { uintptr_t w[3] = { (uintptr_t)counter, (uintptr_t)m, (uintptr_t)total }; drop_state(w); }
    return counter;
}

static inline void dispatch(Dispatcher *d, int64_t k) {
    d->current = k % (int64_t)d->hs.len;
    size_t i = (size_t)d->current;
    e_bounds(i, d->hs.len);
    Callable *slot = &d->hs.ptr[i];
    HandlerFn f = slot->fn;
    RunRec rec = { slot, g_running };
    g_running = &rec;
    slot->fn = running_stub;
    f(&rec, d, k, k % 1000);
    g_running = rec.prev;
    if (rec.slot) rec.slot->fn = f;
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
    for (size_t i = 0; i < d->hs.len; i++) {                     /* for h in d.handlers: s += h(d, 0, -1) */
        Callable *slot = &d->hs.ptr[i];
        HandlerFn f = slot->fn;
        RunRec rec = { slot, g_running };
        g_running = &rec;
        slot->fn = running_stub;
        s += f(&rec, d, 0, -1);
        g_running = rec.prev;
        if (rec.slot) rec.slot->fn = f;
    }
#ifdef BOXED
    int64_t t = total->value;
    h_report("w4-ember-runrec-boxed", (uint64_t)(t + s));
#else
    h_report("w4-ember-runrec", (uint64_t)(total_local + s));
#endif
    return 0;
}
