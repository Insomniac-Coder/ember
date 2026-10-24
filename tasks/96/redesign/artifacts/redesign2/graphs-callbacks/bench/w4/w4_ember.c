/* W4 (callbacks with captured mutable state), Ember under this report's closure and dispatch rules,
   as its compiler would emit this program (Python's version is beside it in the report):

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
                 if v >= 0:                  # v < 0 asks for the counter (for the checksum)
                     counter += v * m
                     total += 1
                     if k % 100_000 == 99_999:
                         d.unregister_current()
                         d.register(make_handler(k % 17 + 1))
                 return counter
             return handler
         ...

   What the rules decide (report section on A12):
   - `counter` is used after `make_handler` returns only by the closure: it moves into the closure.
   - `m` is never written: copied in.
   - `total` is written by the closures and read by `main` afterwards. The closures are stored only in
     `d.handlers`, and `d` is a local of `main` made after `total`, so they die before it: `total` is
     captured as a pointer to `main`'s local (no box). With -DBOXED, `total` lives in a counted box
     instead (what the rules emit when the proof is not available): the closure holds the box.
   - `handler` calls `make_handler`, a local function whose only capture is `total`, which `handler`
     also captures: its captures are merged into `handler`'s (a flat closure), so the state is three
     words, inline in the Array element ([CLO-10]): {counter, m, &total}. With -DHEAPSTATE the merge is
     not done: the state is four words, so it goes to one heap allocation per handler ([CLO-10]).
   - The call `self.handlers[i](...)` runs a closure that may change `self.handlers` (its type says
     nothing about its effects), so the closure is lent out of its slot for the call: moved to the
     caller's frame, a stub left in the slot, moved back after the call if the slot still holds this
     stub, else searched for; if the handler removed its own slot, the moved-out closure is dropped
     after it returns. A call that reaches the stub (the running handler re-entered) panics.
     With -DNOLEND the closure is called in place (NOT safe: a handler that removes itself frees the
     state it is running on). It exists only to take the cost apart.
   - Zero-divisor and overflow checks are left out, as in phase 3's models and plain C. */
#include "../ember_model.h"
#define H 10000
#define E 10000000

typedef struct Dispatcher Dispatcher;
typedef int64_t (*HandlerFn)(uintptr_t *w, Dispatcher *d, int64_t k, int64_t v);
typedef struct { HandlerFn fn; uintptr_t w[3]; } Callable;          /* [CLO-10]: 32 bytes */
typedef struct { Callable *ptr; size_t len, cap; } CVec;
struct Dispatcher { uint32_t rc, type; CVec hs; int64_t current; };

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

static void cvec_push(CVec *v, Callable c) {
    if (v->len == v->cap) { size_t cap = v->cap < 2 ? 4 : v->cap * 2; v->ptr = realloc(v->ptr, cap * sizeof(Callable)); v->cap = cap; }
    v->ptr[v->len++] = c;
}

static int64_t running_stub(uintptr_t *w, Dispatcher *d, int64_t k, int64_t v) {
    (void)w; (void)d; (void)k; (void)v;
    e_panic("this handler is already running (re-entered from its own call)");
    return 0;
}

static void drop_callable(Callable *c);

/* Array.remove(i) on the handler list: drops what it removes (a stub drops nothing). */
static void unregister_current(Dispatcher *d) {
    size_t i = (size_t)d->current;
    e_bounds(i, d->hs.len);
    Callable gone = d->hs.ptr[i];
    memmove(&d->hs.ptr[i], &d->hs.ptr[i + 1], sizeof(Callable) * (d->hs.len - i - 1));
    d->hs.len--;
    if (gone.fn != running_stub) drop_callable(&gone);
}

#ifdef HEAPSTATE
typedef struct { int64_t counter, m; TOTAL_T total; void *make_env; } Env;   /* four words: on the heap */
#define ST(w) ((Env *)(w)[0])
#define COUNTER(w) (ST(w)->counter)
#define M(w) (ST(w)->m)
#define TOTAL(w) (ST(w)->total)
#else
#define COUNTER(w) (*(int64_t *)&(w)[0])
#define M(w) (*(int64_t *)&(w)[1])
#define TOTAL(w) ((TOTAL_T)(w)[2])
#endif

static int64_t handler(uintptr_t *w, Dispatcher *d, int64_t k, int64_t v);
static Callable make_handler(int64_t m, TOTAL_T total) {
    Callable c; c.fn = handler;
    TOTAL_RETAIN(total);
#ifdef HEAPSTATE
    Env *e = malloc(sizeof(Env));
    e->counter = 0; e->m = m; e->total = total; e->make_env = NULL;
    c.w[0] = (uintptr_t)e; c.w[1] = 0; c.w[2] = 0;
#else
    c.w[0] = 0; c.w[1] = (uintptr_t)m; c.w[2] = (uintptr_t)total;
#endif
    return c;
}
static void drop_callable(Callable *c) {
    TOTAL_RELEASE(TOTAL(c->w));
#ifdef HEAPSTATE
    free(ST(c->w));
#endif
}
static void register_handler(Dispatcher *d, Callable c) { cvec_push(&d->hs, c); }

static int64_t handler(uintptr_t *w, Dispatcher *d, int64_t k, int64_t v) {
    if (v >= 0) {
        COUNTER(w) += v * M(w);
        TOTAL_ADD(TOTAL(w));
        int64_t r = COUNTER(w);
        if (k % 100000 == 99999) {
            TOTAL_T t = TOTAL(w);
            unregister_current(d);
            register_handler(d, make_handler(k % 17 + 1, t));
        }
        return r;                     /* nothing of the state is read after the list changed */
    }
    return COUNTER(w);
}

/* put the lent closure back where its stub is now; if the stub is gone, the handler was removed */
static void put_back_slow(Dispatcher *d, Callable *run) {
    for (size_t j = 0; j < d->hs.len; j++)
        if (d->hs.ptr[j].fn == running_stub && d->hs.ptr[j].w[0] == (uintptr_t)run) { d->hs.ptr[j] = *run; return; }
    drop_callable(run);
}

/* How the lent closure is copied. By default, as one struct copy (clang emits 16-byte moves). With
   -DWORDCOPY, word by word, each word through a compiler barrier, so the copy back after the call reads
   each word with the same width the handler wrote it: the handler's 8-byte store of `counter` can then
   be forwarded to the load (a 16-byte load of a just-stored 8-byte word stalls on x86). */
#ifdef WORDCOPY
#define COPY_CALLABLE(dst, src) do {     uintptr_t *d_ = (uintptr_t *)(dst); const uintptr_t *s_ = (const uintptr_t *)(src);     uintptr_t a_ = s_[0]; __asm__ volatile("" : "+r"(a_)); uintptr_t b_ = s_[1]; __asm__ volatile("" : "+r"(b_));     uintptr_t c_ = s_[2]; __asm__ volatile("" : "+r"(c_)); uintptr_t e_ = s_[3]; __asm__ volatile("" : "+r"(e_));     d_[0] = a_; d_[1] = b_; d_[2] = c_; d_[3] = e_; } while (0)
#else
#define COPY_CALLABLE(dst, src) (*(dst) = *(src))
#endif

static inline void dispatch(Dispatcher *d, int64_t k) {
    d->current = k % (int64_t)d->hs.len;
    size_t i = (size_t)d->current;
    e_bounds(i, d->hs.len);
#ifdef NOLEND
    Callable *c = &d->hs.ptr[i];
    c->fn(c->w, d, k, k % 1000);
#else
    Callable run;                                                /* lend the closure out */
    COPY_CALLABLE(&run, &d->hs.ptr[i]);
    d->hs.ptr[i].fn = running_stub; d->hs.ptr[i].w[0] = (uintptr_t)&run;
    run.fn(run.w, d, k, k % 1000);
    if (i < d->hs.len && d->hs.ptr[i].fn == running_stub && d->hs.ptr[i].w[0] == (uintptr_t)&run)
        COPY_CALLABLE(&d->hs.ptr[i], &run);                      /* the common case: one compare pair */
    else
        put_back_slow(d, &run);
#endif
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
        Callable *c = &d->hs.ptr[i];
        s += c->fn(c->w, d, 0, -1);
    }
#ifdef BOXED
    int64_t t = total->value;
#else
    int64_t t = total_local;
#endif
#if defined(NOLEND)
    const char *name = "w4-ember-nolend";
#elif defined(WORDCOPY) && defined(HEAPSTATE)
    const char *name = "w4-ember-wordcopy-heapstate";
#elif defined(WORDCOPY) && defined(BOXED)
    const char *name = "w4-ember-wordcopy-boxed";
#elif defined(WORDCOPY)
    const char *name = "w4-ember-wordcopy";
#elif defined(HEAPSTATE) && defined(BOXED)
    const char *name = "w4-ember-heapstate-boxed";
#elif defined(HEAPSTATE)
    const char *name = "w4-ember-heapstate";
#elif defined(BOXED)
    const char *name = "w4-ember-boxed";
#else
    const char *name = "w4-ember";
#endif
    h_report(name, (uint64_t)(t + s));
    return 0;
}
