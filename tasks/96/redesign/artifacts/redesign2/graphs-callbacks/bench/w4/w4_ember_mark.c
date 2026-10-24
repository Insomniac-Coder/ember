/* W4, Ember: the same program and closure rules as w4_ember.c, with a third way to make the call of a
   stored closure safe when the call may change the list that holds it ("running mark"):

   - A closure value stored in an Array keeps its captured state in one heap block (what plain C's
     `env` and Rust's Box<dyn FnMut> do): the element is {function, state pointer}, 16 bytes. Moving
     the element (the Array grows, an element before it is removed) moves only these two words; the
     state never moves. The block is made when the closure is stored (here: in `make_handler`, once
     per registration).
   - The state block starts with a flags word. The call sets "running" before and clears it after.
     A call that finds "running" already set panics (the closure re-entered from its own call).
   - Dropping a stored closure whose state is "running" (its slot was removed, the list cleared or
     replaced during the call) only marks it "removed"; the caller frees it when the call returns.
   With -DBOXED, `total` is in a counted box, as in w4_ember.c. */
#include "../ember_model.h"
#define H 10000
#define E 10000000

typedef struct Dispatcher Dispatcher;
#define RUNNING 1u
#define REMOVED 2u
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
typedef struct { uint64_t flags; int64_t counter, m; TOTAL_T total; } Env;
typedef int64_t (*HandlerFn)(Env *e, Dispatcher *d, int64_t k, int64_t v);
typedef struct { HandlerFn fn; Env *env; } Callable;
typedef struct { Callable *ptr; size_t len, cap; } CVec;
struct Dispatcher { uint32_t rc, type; CVec hs; int64_t current; };

static void cvec_push(CVec *v, Callable c) {
    if (v->len == v->cap) { size_t cap = v->cap < 2 ? 4 : v->cap * 2; v->ptr = realloc(v->ptr, cap * sizeof(Callable)); v->cap = cap; }
    v->ptr[v->len++] = c;
}
static void drop_env(Env *e) { TOTAL_RELEASE(e->total); free(e); }
static void drop_callable(Callable *c) {
    if (c->env->flags & RUNNING) c->env->flags |= REMOVED;      /* the running call frees it */
    else drop_env(c->env);
}
static void unregister_current(Dispatcher *d) {
    size_t i = (size_t)d->current;
    e_bounds(i, d->hs.len);
    Callable gone = d->hs.ptr[i];
    memmove(&d->hs.ptr[i], &d->hs.ptr[i + 1], sizeof(Callable) * (d->hs.len - i - 1));
    d->hs.len--;
    drop_callable(&gone);
}
static int64_t handler(Env *e, Dispatcher *d, int64_t k, int64_t v);
static Callable make_handler(int64_t m, TOTAL_T total) {
    TOTAL_RETAIN(total);
    Env *e = malloc(sizeof(Env));
    e->flags = 0; e->counter = 0; e->m = m; e->total = total;
    return (Callable){ handler, e };
}
static void register_handler(Dispatcher *d, Callable c) { cvec_push(&d->hs, c); }

static int64_t handler(Env *e, Dispatcher *d, int64_t k, int64_t v) {
    if (v >= 0) {
        e->counter += v * e->m;
        TOTAL_ADD(e->total);
        if (k % 100000 == 99999) {
            unregister_current(d);
            register_handler(d, make_handler(k % 17 + 1, e->total));
        }
    }
    return e->counter;
}

static inline int64_t call_stored(Callable c, Dispatcher *d, int64_t k, int64_t v) {
    uint64_t f = c.env->flags;
    if (f & RUNNING) e_panic("this handler is already running (re-entered from its own call)");
    c.env->flags = f | RUNNING;
    int64_t r = c.fn(c.env, d, k, v);
    f = c.env->flags;
    if (f & REMOVED) drop_env(c.env); else c.env->flags = f & ~RUNNING;
    return r;
}

static inline void dispatch(Dispatcher *d, int64_t k) {
    d->current = k % (int64_t)d->hs.len;
    size_t i = (size_t)d->current;
    e_bounds(i, d->hs.len);
    call_stored(d->hs.ptr[i], d, k, k % 1000);
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
    for (size_t i = 0; i < d->hs.len; i++) s += call_stored(d->hs.ptr[i], d, 0, -1);
#ifdef BOXED
    h_report("w4-ember-mark-boxed", (uint64_t)(total->value + s));
#else
    h_report("w4-ember-mark", (uint64_t)(total_local + s));
#endif
    return 0;
}
