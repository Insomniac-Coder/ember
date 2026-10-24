/* attacks2.c: programs the graphs-and-callbacks group's rules accept, written as the C their rules
   would make the compiler emit, each in two forms: as the rule is written, and with the fix proposed
   in phase3b-graphs-callbacks-attack.md. Built with AddressSanitizer so a read or write of freed or
   out-of-range memory stops the program with a report:

     "C:/Program Files/LLVM/bin/clang.exe" -O1 -g -fsanitize=address -I ../../../redesign/bench attacks2.c -o attacks2.exe
     attacks2.exe <mode>     (odd modes: as written; the even mode after each: fixed)

   Mode 1/2   isinstance narrowing of a local that a stored closure writes (A13 with A12)
   Mode 3/4   a stored closure called with an argument whose evaluation removes it (A32 rule 2)
   Mode 5/6   a closure lending its own captured list to a call that frees its box (A32 rule 3)
   Mode 7/8   a one-to-one back link assigned the value it already holds (A8 rules 2 and 3)
   Mode 9/10  Array[weak T] registered by slot address, after a removal shifts the slots (A8 weak)
   Mode 11/12 a list captured by pointer, emptied by a closure while a loop walks it (A12)
   Mode 13/14 the fix of 9/10 attacked again: a weak list registered by its header, moved out of its
              field (13); registered by its buffer instead (14)

   h_start() (harness.h) calls SetErrorMode first, so nothing can open a dialog. */
#include "harness.h"
#include <string.h>

typedef struct { uint32_t rc, type; } Hdr;   /* the lean class header: count, type index */
static void panic(const char *msg) { printf("panic: %s\n", msg); fflush(stdout); exit(101); }

/* ---------------------------------------------------------------------------------------------- */
/* Mode 1/2. Ember (proposed rules):
       open class Shape: pass
       class Circle(Shape): r: float = 1.5
       class Square(Shape): pass
       fn main():
           s: Shape = Circle()
           hs = [fn(): s = Square()]        # a closure that writes s: s lives in a box (A12 rule 2)
           if isinstance(s, Circle):        # narrows s to Circle "until the local is assigned"
               hs[0]()                      # no assignment to s is written here
               println(s.r)                 # as written: still narrowed                          */
enum { T_CIRCLE = 1, T_SQUARE = 2 };
typedef struct { Hdr h; double r; } Circle;
typedef struct { Hdr h; } Square;
typedef struct { uint32_t rc; Hdr *v; } ShapeBox;            /* the box holding the local `s` */
typedef struct { uint32_t rc; ShapeBox *s; } ResetState;     /* the closure's state: s's box */
typedef struct { void (*fn)(void *); void *st; } Callable0;
static void reset_body(void *stv) {                           /* fn(): s = Square() */
    ResetState *st = stv;
    Square *sq = calloc(1, sizeof *sq); sq->h.rc = 1; sq->h.type = T_SQUARE;
    Hdr *old = st->s->v;
    st->s->v = &sq->h;
    if (--old->rc == 0) free(old);
}
static void mode_narrow(int fixed) {
    ShapeBox *s = malloc(sizeof *s); s->rc = 1;
    Circle *c = calloc(1, sizeof *c); c->h.rc = 1; c->h.type = T_CIRCLE; c->r = 1.5;
    s->v = &c->h;
    ResetState *rs = malloc(sizeof *rs); rs->rc = 1; rs->s = s; s->rc++;
    Callable0 hs[1] = {{reset_body, rs}};
    if (s->v->type == T_CIRCLE) {                             /* isinstance(s, Circle) */
        hs[0].fn(hs[0].st);                                   /* hs[0]() */
        if (!fixed) {
            printf("s.r = %f\n", ((Circle *)s->v)->r);        /* narrowed read: s is a Square now */
        } else {
            /* fixed: s is written by a closure, so a call that may run that closure ends the
               narrowing; `s.r` here is a compile-time error asking for the test again: */
            if (s->v->type == T_CIRCLE) printf("s.r = %f\n", ((Circle *)s->v)->r);
            else printf("s is no longer a Circle after hs[0](); the test is asked for again\n");
        }
    }
}

/* ---------------------------------------------------------------------------------------------- */
/* Mode 3/4. Ember:
       class Bus:
           handlers: Array[fn(int)] = []
           fn clear_and_count(self) -> int:
               n = len(self.handlers); self.handlers.clear(); return n
       fn main():
           bus = Bus(); name = "first"
           bus.handlers.push(fn(n: int): println(f"{name} {n}"))
           bus.handlers[0](bus.clear_and_count())   # Python: the callee is evaluated first
   The report's A32 rule 2: "calling a stored closure copies {function, box} out of its slot and
   calls. The caller uses nothing of it after the call, so it protects nothing." */
typedef struct { void (*fn)(void *st, int64_t n); void *st; } Callable1;
typedef struct { Hdr h; Callable1 *ptr; size_t len, cap; } Bus;
typedef struct { uint32_t rc; char *name; } NameState;
static void name_release(NameState *s) { if (--s->rc == 0) { free(s->name); free(s); } }
static void print_body(void *stv, int64_t n) { NameState *st = stv; printf("%s %lld\n", st->name, (long long)n); }
static int64_t clear_and_count(Bus *b) {
    int64_t n = (int64_t)b->len;
    for (size_t i = 0; i < b->len; i++) name_release(b->ptr[i].st);
    b->len = 0;
    return n;
}
static void mode_callee_args(int fixed) {
    Bus *bus = calloc(1, sizeof *bus); bus->h.rc = 1;
    NameState *ns = malloc(sizeof *ns); ns->rc = 1; ns->name = _strdup("first");
    bus->ptr = malloc(4 * sizeof(Callable1)); bus->cap = 4;
    bus->ptr[bus->len++] = (Callable1){print_body, ns};
    if (0 >= bus->len) panic("index out of range");
    Callable1 c = bus->ptr[0];                       /* the callee, copied out of its slot */
    if (fixed) ((NameState *)c.st)->rc++;            /* fixed: used (by the call) after a call that may release it */
    int64_t a = clear_and_count(bus);                /* the argument: removes every handler */
    c.fn(c.st, a);
    if (fixed) name_release(c.st);
}

/* ---------------------------------------------------------------------------------------------- */
/* Mode 5/6. Ember:
       class Bus2:
           handlers: Array[fn(Bus2)] = []
           fn report(self, xs: Array[int]):         # a read-only loan of the caller's list
               self.handlers.clear()                # drops the running handler's box
               println(xs[0])
       fn main():
           bus = Bus2()
           seen = [1, 2, 3]                         # used after this only by the closure: moved into its box
           bus.handlers.push(fn(b: Bus2): b.report(seen))
           bus.handlers[0](bus)
   The report's A32 rule 3: the body retains its box across a call that may release it when it
   "uses its box after it". This body uses nothing after the call; it lends `seen` *to* the call. */
typedef struct Bus2 Bus2;
typedef struct { void (*fn)(void *st, Bus2 *b); void *st; } Callable2;
struct Bus2 { Hdr h; Callable2 *ptr; size_t len, cap; };
typedef struct { uint32_t rc; int64_t *seen; size_t len; } SeenState;
static void seen_release(SeenState *s) { if (--s->rc == 0) { free(s->seen); free(s); } }
static void report(Bus2 *b, const int64_t *xs, size_t n) {
    for (size_t i = 0; i < b->len; i++) seen_release(b->ptr[i].st);
    b->len = 0;
    if (0 >= n) panic("index out of range");
    printf("xs[0] = %lld\n", (long long)xs[0]);
}
static int g_fixed;
static void lend_body(void *stv, Bus2 *b) {
    SeenState *st = stv;
    if (g_fixed) st->rc++;                           /* fixed: a loan into the box crosses a call that may release it */
    report(b, st->seen, st->len);
    if (g_fixed) seen_release(st);
}
static void mode_lent_state(int fixed) {
    g_fixed = fixed;
    Bus2 *bus = calloc(1, sizeof *bus); bus->h.rc = 1;
    SeenState *ss = malloc(sizeof *ss); ss->rc = 1; ss->len = 3;
    ss->seen = malloc(3 * sizeof(int64_t)); ss->seen[0] = 1; ss->seen[1] = 2; ss->seen[2] = 3;
    bus->ptr = malloc(4 * sizeof(Callable2)); bus->cap = 4;
    bus->ptr[bus->len++] = (Callable2){lend_body, ss};
    Callable2 c = bus->ptr[0];
    c.fn(c.st, bus);
}

/* ---------------------------------------------------------------------------------------------- */
/* Mode 7/8. Ember (a one-to-one back link, the report's doubly linked list):
       class LNode:
           v: int
           next: Option[LNode] = None
           prev: Option[LNode] = backlink(next)
       a.next = a.next           # or any `x.next = y` where y already is x.next
   Rule 2: "every operation that puts an object into g first takes the object out of its current
   holder's g, then sets its back link"; rule 3: "every operation that takes an object out of g
   (... replacing the whole field) sets its back link to None". The usual code for an assignment
   reads the old value first, stores, then releases the old value. */
typedef struct LNode { Hdr h; int64_t v; struct LNode *next, *prev; } LNode;
static void lnode_release(LNode *n) {
    if (--n->h.rc == 0) {
        if (n->next) { n->next->prev = NULL; lnode_release(n->next); }   /* rule 4 */
        free(n);
    }
}
static LNode *lnode_new(int64_t v) { LNode *n = calloc(1, sizeof *n); n->h.rc = 1; n->v = v; return n; }
static void set_next_as_written(LNode *a, LNode *val) {
    LNode *old = a->next;                                   /* the old value, read first */
    if (val && val->prev) val->prev->next = NULL;           /* rule 2: out of its holder; its count moves */
    a->next = val; if (val) val->prev = a;
    if (old) { old->prev = NULL; lnode_release(old); }      /* rule 3: what was replaced */
}
static void set_next_fixed(LNode *a, LNode *val) {
    LNode *old = a->next;
    if (old) old->prev = NULL;                              /* 1. unlink the old value */
    if (val) {
        if (val->prev) val->prev->next = NULL;              /* 2. still in a holder: the count moves */
        else val->h.rc++;                                   /*    in none: a new owning place */
        val->prev = a;
    }
    a->next = val;
    if (old) lnode_release(old);                            /* 3. release the old value */
}
static void mode_link_self_assign(int fixed) {
    LNode *a = lnode_new(1), *c = lnode_new(2);
    a->next = c; c->prev = a;                               /* a.next = c (c's only owner is a.next) */
    if (fixed) set_next_fixed(a, a->next); else set_next_as_written(a, a->next);
    printf("a.next.v = %lld\n", (long long)a->next->v);     /* Python: 2, nothing changed */
    lnode_release(a);
}

/* ---------------------------------------------------------------------------------------------- */
/* Mode 9/10. Ember (a `weak` list, cleared by registration):
       class Subject:
           observers: Array[weak Observer] = []
       s.observers.push(o1); ... push(o3)
       s.observers.remove(0)              # shifts o2, o3 down one slot
       s.observers.push(o4); push(o5); push(o6)   # the 5th element moves the buffer
       o2 = None                          # o2 dies: every registered field is set to None
   The report: "storing into a weak field registers the field's address with the target ... an Array
   of weak elements re-registers its elements when its buffer moves". A removal moves elements too.
   Each target's record is reached here through a pointer kept beside the demo object; the real side
   table is keyed by the object's address (w2_zweak.c); which addresses it holds is what matters. */
typedef struct Obs { Hdr h; int id; struct Rec *rec; } Obs;
typedef struct Rec { void *regs[16]; int n; } Rec;           /* registered addresses (slots or arrays) */
typedef struct { Obs **ptr; size_t len, cap; } WeakArr;
typedef struct { Hdr h; WeakArr observers; } Subject;
static Obs *obs_new(int id) { Obs *o = calloc(1, sizeof *o); o->h.rc = 1; o->id = id; o->rec = calloc(1, sizeof(Rec)); return o; }
static void rec_add(Obs *o, void *addr) { o->rec->regs[o->rec->n++] = addr; }
static void rec_remove(Obs *o, void *addr) {
    for (int i = 0; i < o->rec->n; i++) if (o->rec->regs[i] == addr) { o->rec->regs[i] = o->rec->regs[--o->rec->n]; return; }
}
static void rec_replace(Obs *o, void *from, void *to) {
    for (int i = 0; i < o->rec->n; i++) if (o->rec->regs[i] == from) { o->rec->regs[i] = to; return; }
}
static int g_by_array;
static void wa_push(WeakArr *a, Obs *o) {
    if (a->len == a->cap) {                                  /* the buffer moves */
        size_t nc = a->cap ? a->cap * 2 : 4;
        Obs **nb = malloc(nc * sizeof *nb);
        memcpy(nb, a->ptr, a->len * sizeof *nb);
        if (!g_by_array)                                     /* as written: re-register each element */
            for (size_t i = 0; i < a->len; i++) if (nb[i]) rec_replace(nb[i], &a->ptr[i], &nb[i]);
        free(a->ptr); a->ptr = nb; a->cap = nc;
    }
    a->ptr[a->len] = o;
    rec_add(o, g_by_array ? (void *)a : (void *)&a->ptr[a->len]);
    a->len++;
}
static void wa_remove(WeakArr *a, size_t k) {
    if (k >= a->len) panic("index out of range");
    if (a->ptr[k]) rec_remove(a->ptr[k], g_by_array ? (void *)a : (void *)&a->ptr[k]);
    memmove(&a->ptr[k], &a->ptr[k + 1], (a->len - k - 1) * sizeof *a->ptr);
    a->len--;
}
static void obs_die(Obs *o) {                                /* the target dies: clear what is registered */
    for (int i = 0; i < o->rec->n; i++) {
        if (!g_by_array) *(Obs **)o->rec->regs[i] = NULL;
        else { WeakArr *a = o->rec->regs[i]; for (size_t k = 0; k < a->len; k++) if (a->ptr[k] == o) a->ptr[k] = NULL; }
    }
    free(o->rec); free(o);
}
static void mode_weak_array(int fixed) {
    g_by_array = fixed;
    Subject *s = calloc(1, sizeof *s); s->h.rc = 1;
    Obs *o[7]; for (int i = 1; i <= 6; i++) o[i] = obs_new(i);
    wa_push(&s->observers, o[1]); wa_push(&s->observers, o[2]); wa_push(&s->observers, o[3]);
    wa_remove(&s->observers, 0);
    wa_push(&s->observers, o[4]); wa_push(&s->observers, o[5]); wa_push(&s->observers, o[6]);
    obs_die(o[2]);
    printf("observers:");
    for (size_t k = 0; k < s->observers.len; k++) {
        if (s->observers.ptr[k]) printf(" %d", s->observers.ptr[k]->id); else printf(" None");
    }
    printf("\n");
}

/* ---------------------------------------------------------------------------------------------- */
/* Mode 11/12. Ember:
       fn main():
           xs = ["ab", "cde", "f"]
           hs = [fn(): xs.clear()]          # hs dies before xs: xs is captured by pointer (A12 rule 2)
           total = 0
           for x in xs:
               hs[0]()                      # runs xs.clear()
               total += len(x)
           println(total)
   The first attack (its section 5.1) made a *boxed* variable a heap place in effect summaries; this
   variable is not boxed, so that fix does not reach it. */
typedef struct { char **ptr; size_t len, cap; uint8_t walkers; } StrList;
typedef struct { StrList *xs; } ClearState;                  /* captured by pointer */
static void strlist_clear(StrList *l) {
    if (l->walkers) panic("xs changed while a loop over it was running (line 6)");
    for (size_t i = 0; i < l->len; i++) free(l->ptr[i]);
    l->len = 0;
}
static void clear_body(void *stv) { ClearState *st = stv; strlist_clear(st->xs); }
static void mode_pointer_capture(int fixed) {
    StrList xs = {0};
    xs.ptr = malloc(4 * sizeof(char *)); xs.cap = 4;
    xs.ptr[xs.len++] = _strdup("ab"); xs.ptr[xs.len++] = _strdup("cde"); xs.ptr[xs.len++] = _strdup("f");
    ClearState cs = {&xs};
    Callable0 hs[1] = {{clear_body, &cs}};
    int64_t total = 0;
    if (!fixed) {
        /* as written: nothing in the loop's own text changes xs, so the loop keeps its pointer and length */
        char **p = xs.ptr; size_t n = xs.len;
        for (size_t i = 0; i < n; i++) { char *x = p[i]; hs[0].fn(hs[0].st); total += (int64_t)strlen(x); }
    } else {
        /* fixed: xs is written by a closure, so the call is an unseen change of xs: the containers
           group's rule 9 (walkers count; positions re-read after the call) */
        xs.walkers++;
        for (size_t i = 0; i < xs.len; i++) { char *x = xs.ptr[i]; hs[0].fn(hs[0].st); total += (int64_t)strlen(x); }
        xs.walkers--;
    }
    printf("total = %lld\n", (long long)total);
}

/* ---------------------------------------------------------------------------------------------- */
/* Mode 13/14. The first save of modes 9/10 (register the list's own {ptr, len, cap} with each target),
   attacked again: the list value can move out of its field ("names follow values",
   phase3b-ownership.md 2.2 rule 2: the old value moves into the name's own storage).
       s.observers.push(o1)
       old = s.observers          # `old` names the list ...
       s.observers = []           # ... which moves into `old` here instead of being dropped
       o1 = None                  # o1 dies
       println(old[0])            # Python: None
   Mode 13 registers &s.observers (the header, which stayed in s); mode 14 registers the buffer
   (with a one-word prefix holding the live length), which moves with the value. */
typedef struct { size_t len; } BufHdr;
static void mode_weak_list_moved(int by_buffer) {
    Subject *s = calloc(1, sizeof *s); s->h.rc = 1;
    Obs *o1 = obs_new(1);
    BufHdr *bh = malloc(sizeof(BufHdr) + 4 * sizeof(Obs *));
    s->observers.ptr = (Obs **)(bh + 1); s->observers.cap = 4;
    s->observers.ptr[0] = o1; s->observers.len = 1; bh->len = 1;
    rec_add(o1, by_buffer ? (void *)bh : (void *)&s->observers);
    WeakArr old = s->observers;                              /* the value moves into `old` */
    s->observers = (WeakArr){0};                             /* s.observers = [] */
    for (int i = 0; i < o1->rec->n; i++) {                   /* o1 dies */
        if (by_buffer) {
            BufHdr *h = o1->rec->regs[i]; Obs **slots = (Obs **)(h + 1);
            for (size_t k = 0; k < h->len; k++) if (slots[k] == o1) slots[k] = NULL;
        } else {
            WeakArr *a = o1->rec->regs[i];
            for (size_t k = 0; k < a->len; k++) if (a->ptr[k] == o1) a->ptr[k] = NULL;
        }
    }
    free(o1->rec); free(o1);
    if (old.ptr[0]) printf("old[0].id = %d\n", old.ptr[0]->id); else printf("old[0] = None\n");
}

int main(int argc, char **argv) {
    h_start();
    int mode = argc > 1 ? atoi(argv[1]) : 0;
    int fixed = (mode % 2) == 0;
    switch ((mode + 1) / 2) {
    case 1: mode_narrow(fixed); break;
    case 2: mode_callee_args(fixed); break;
    case 3: mode_lent_state(fixed); break;
    case 4: mode_link_self_assign(fixed); break;
    case 5: mode_weak_array(fixed); break;
    case 6: mode_pointer_capture(fixed); break;
    case 7: mode_weak_list_moved(fixed); break;
    default: printf("usage: attacks2.exe <1..14>\n"); return 2;
    }
    return 0;
}
