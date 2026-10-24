/* attacks.c: what the graphs-and-callbacks answers' rules, as written, would make the compiler emit
   for five programs those rules accept, and the same programs under the fixes this attack proposes.
   Each mode models one Ember program (shown in the comment above it). Built with AddressSanitizer:

     "C:/Program Files/LLVM/bin/clang.exe" -O1 -g -fsanitize=address -I ../../../redesign/bench attacks.c -o attacks.exe
     ASAN_OPTIONS=abort_on_error=0 ./attacks.exe <mode>

   Modes: 1 back-link element assignment as written, 2 its fix, 3 Python's swap idiom under the safe
   reading of the same rule; 4 `weak` by registration as written, 5 its fix; 6 isinstance narrowing as
   written, 7 its fix; 8 a boxed captured variable walked by a loop as written, 9 its fix; 10 a closure
   overwritten during its own call as written, 11 its fix. h_start() calls SetErrorMode first, so no
   dialog can open. */
#include "harness.h"
#include <string.h>

static void panic_msg(const char *m) { printf("panic: %s\n", m); fflush(stdout); exit(101); }

/* ---------- A8 back links: a node, its children list, its back link ---------- */
typedef struct Node {
    uint32_t rc, type;                 /* lean header: count, type index (1 Node, 2 Folder) */
    const char *name;
    struct Node *parent;               /* parent: Option[Node] = backlink(children) */
    struct Node **ch; size_t len, cap; /* children: Array[Node] */
} Node;
typedef struct Folder { Node base; const char *icon; } Folder;   /* class Folder(Node): icon: String */

static Node *node_new(const char *name) {
    Node *n = calloc(1, sizeof(Node)); n->rc = 1; n->type = 1; n->name = name; return n;
}
static Node *folder_new(const char *name, const char *icon) {
    Folder *f = calloc(1, sizeof(Folder)); f->base.rc = 1; f->base.type = 2; f->base.name = name; f->icon = icon;
    return &f->base;
}
static void release(Node *n) {
    if (--n->rc) return;
    for (size_t i = 0; i < n->len; i++) n->ch[i]->parent = NULL;   /* rule 4: clear, then release */
    for (size_t i = 0; i < n->len; i++) release(n->ch[i]);
    free(n->ch); free(n);
}
static void ch_push_raw(Node *h, Node *c) {
    if (h->len == h->cap) { h->cap = h->cap ? h->cap * 2 : 4; h->ch = realloc(h->ch, h->cap * sizeof *h->ch); }
    h->ch[h->len++] = c;
}
static size_t ch_index_of(Node *h, Node *c) { for (size_t k = 0; k < h->len; k++) if (h->ch[k] == c) return k; panic_msg("not in list"); return 0; }
static void ch_take_out(Node *c) {        /* rule 2's first step: out of the old holder, no count change */
    Node *old = c->parent; if (!old) return;
    size_t k = ch_index_of(old, c);
    memmove(&old->ch[k], &old->ch[k + 1], (old->len - k - 1) * sizeof *old->ch);
    old->len--;
    c->parent = NULL;
}
static void children_push(Node *h, Node *c) { ch_take_out(c); ch_push_raw(h, c); c->parent = h; }
static void print_children(Node *p) {
    printf("%s.children = [", p->name);
    for (size_t i = 0; i < p->len; i++) printf("%s%s", i ? ", " : "", p->ch[i]->name);
    printf("]\n");
}

/* Mode 1, as written (slot found, then rule 2), and mode 3 (rule 2 first, then the slot):
       p.children = [a, b]
       p.children[1] = p.children[0]          # Python: [a, a]
   Rule 2 says an assignment of an element first takes the object out of its current holder. If the
   slot's address was computed (and bounds-checked) before that, the store lands at the old index 1
   after a has been taken out and b shifted down: it releases the stale copy of b left there, so b is
   freed while p.children[0] still holds it. */
static void a8_assign_slot_first(void) {
    Node *p = node_new("p"), *a = node_new("a"), *b = node_new("b");
    children_push(p, a); children_push(p, b);
    Node *x = p->ch[0]; x->rc++;                    /* the right-hand side, retained while used */
    size_t i = 1; if (i >= p->len) panic_msg("index out of range");
    Node **slot = &p->ch[i];                        /* the place, evaluated first */
    ch_take_out(x);                                 /* rule 2: a leaves p.children: [b] */
    Node *old = *slot; *slot = x; x->parent = p;    /* stores past the live length */
    old->parent = NULL; release(old);               /* rule 3 on the element it replaced: frees b */
    x->rc--;
    printf("len %zu, first child: ", p->len); fflush(stdout);
    printf("%s\n", p->ch[0]->name);                 /* b: freed */
}
/* Mode 2, fixed: an assignment whose value is already an element of the same list is a move inside
   that list only through swap(): `xs[i], xs[j] = xs[j], xs[i]` compiles to xs.swap(i, j); any other
   such assignment panics with a message naming both positions (the value's own slot is a no-op). The
   test is one compare on the path rule 2 already takes (c->parent is loaded there anyway). */
static void ch_assign(Node *h, size_t i, Node *c) {
    if (i >= h->len) panic_msg("index out of range");
    if (c->parent == h) {
        if (h->ch[i] == c) return;
        printf("panics here (the demo goes on): %s is already at children[%zu]; a node has one place: use swap(), insert() or push()\n",
               c->name, ch_index_of(h, c));
        return;
    }
    ch_take_out(c);
    Node *old = h->ch[i]; h->ch[i] = c; c->parent = h;           /* store first, then drop (ownership 2.2) */
    old->parent = NULL; release(old);
}
static void ch_swap(Node *h, size_t i, size_t j) {
    if (i >= h->len || j >= h->len) panic_msg("index out of range");
    Node *t = h->ch[i]; h->ch[i] = h->ch[j]; h->ch[j] = t;
}
static void a8_assign_fixed(void) {
    Node *p = node_new("p"), *a = node_new("a"), *b = node_new("b"), *c = node_new("c");
    children_push(p, a); children_push(p, b);
    ch_assign(p, 1, p->ch[0]);
    children_push(p, c);
    ch_swap(p, 0, 2);                               /* p.children[0], p.children[2] = p.children[2], p.children[0] */
    print_children(p);
    release(p);
}
/* Mode 3: the safe reading of rule 2 (take out first, then find and check the slot) on Python's swap:
       p.children = [a, b, c]
       p.children[0], p.children[2] = p.children[2], p.children[0]     # Python: [c, b, a]  */
static void a8_swap_rule2(void) {
    Node *p = node_new("p"), *a = node_new("a"), *b = node_new("b"), *c = node_new("c");
    children_push(p, a); children_push(p, b); children_push(p, c);
    Node *r0 = p->ch[2], *r1 = p->ch[0]; r0->rc++; r1->rc++;     /* right-hand side first */
    Node *tgt[2] = { r0, r1 }; size_t idx[2] = { 0, 2 };
    for (int s = 0; s < 2; s++) {
        Node *x = tgt[s];
        ch_take_out(x);
        print_children(p);
        if (idx[s] >= p->len) panic_msg("index out of range");
        Node *old = p->ch[idx[s]]; p->ch[idx[s]] = x; x->parent = p;
        old->parent = NULL; release(old);
    }
    print_children(p);
}

/* ---------- A8 `weak` cleared by registration ---------- */
typedef struct Obj { uint32_t rc; struct Obj **regs[8]; int nregs; } Obj;     /* the target's side record */
typedef struct Camera { uint32_t rc; Obj *target; } Camera;                    /* class Camera: target: weak Player */
static void weak_store(Obj **field, Obj *t) {
    Obj *old = *field;
    if (old) for (int i = 0; i < old->nregs; i++) if (old->regs[i] == field) { old->regs[i] = old->regs[--old->nregs]; break; }
    if (t) t->regs[t->nregs++] = field;
    *field = t;
}
static void obj_release(Obj *o) {
    if (--o->rc) return;
    for (int i = 0; i < o->nregs; i++) *o->regs[i] = NULL;      /* the death clears every registered field */
    free(o);
}
/* Mode 4, as written:
       player = Player(); cam = Camera(); cam.target = player
       cam = None            # the holder dies first: its field stays registered with player
       player = None         # player's death clears the registered field: a write into freed memory */
static void weak_camera_drop(Camera *c, int deregister) {
    if (deregister && c->target) weak_store(&c->target, NULL);   /* the fix: a holder's drop stores None */
    free(c);
}
static void a8_weak(int fixed) {
    Obj *player = calloc(1, sizeof(Obj)); player->rc = 1;
    Camera *cam = calloc(1, sizeof(Camera)); cam->rc = 1;
    weak_store(&cam->target, player);
    weak_camera_drop(cam, fixed);
    obj_release(player);
    printf("player and camera both gone; registered fields left: none written after free\n");
}

/* ---------- A13 narrowing a back link across an implicit write ---------- */
/* Mode 6, as written:
       if isinstance(n.parent, Folder):     # narrowed: n.parent is a Folder in this block
           other.children.push(n)           # rule 2 sets n.parent = other; the source writes no `parent`
           print(n.parent.icon)             # still narrowed if summaries list only written fields
   Mode 7, fixed: every operation of rule 2, 3 and 4 is in summaries as a write of `Node.parent`, so the
   push ends the narrowing; the program must test again. */
static void a13_narrow(int fixed) {
    Node *root = node_new("root"), *docs = folder_new("docs", "folder.png"), *other = node_new("other");
    children_push(root, docs); children_push(root, other);
    Node *n = node_new("n"); children_push(docs, n);
    if (n->parent && n->parent->type == 2) {        /* isinstance(n.parent, Folder) */
        children_push(other, n);
        if (fixed && !(n->parent && n->parent->type == 2)) { printf("n.parent is %s, not a Folder\n", n->parent->name); return; }
        printf("icon: "); fflush(stdout);
        printf("%s\n", ((Folder *)n->parent)->icon); /* other is a Node: read past its block */
    }
}

/* ---------- A12 a variable kept in a box, walked by a loop that calls unknown code ---------- */
typedef struct { char *s; } Str;
typedef struct { uint32_t rc; Str *ptr; size_t len, cap; uint8_t walkers; } XsBox;   /* the box holding `xs` */
static XsBox *g_xs; static void (*g_on_reset)(void);
static void reset_closure(void) {                   /* bus.on("reset", fn(): xs.clear()) */
    if (g_xs->walkers) { printf("panic: xs changed while a loop over it was running (line 6)\n"); fflush(stdout); exit(101); }
    for (size_t i = 0; i < g_xs->len; i++) free(g_xs->ptr[i].s);
    g_xs->len = 0;
}
static __attribute__((noinline)) void bus_emit(void) { g_on_reset(); }   /* code the loop cannot see */
/* Mode 8, as written:
       xs = ["ab", "cde", "f"]
       bus.on("reset", fn(): xs.clear())   # xs is captured by a stored closure: it lives in a box
       for x in xs:                         # if the box is treated as a plain local, nothing else
           bus.emit("reset")                #   "can" change xs: the loop keeps a pointer and a length
           total += len(x)
   Mode 9, fixed: a variable in a box is a heap place in effect summaries, reachable by every closure
   that captures it; a call with unknown effects may reshape it, so the containers group's rule 9
   applies (positions, a walkers count, a panic if unseen code removes). */
static void a12_boxed_loop(int fixed) {
    g_xs = calloc(1, sizeof(XsBox)); g_xs->rc = 2;
    const char *init[3] = { "ab", "cde", "f" };
    g_xs->cap = 4; g_xs->ptr = calloc(4, sizeof(Str));
    for (int i = 0; i < 3; i++) { g_xs->ptr[i].s = _strdup(init[i]); g_xs->len++; }
    g_on_reset = reset_closure;
    size_t total = 0;
    if (!fixed) {
        Str *p = g_xs->ptr; size_t n = g_xs->len;   /* an unaliased local: pointer and length kept */
        for (size_t i = 0; i < n; i++) { bus_emit(); total += strlen(p[i].s); }
    } else {
        g_xs->walkers++;                            /* rule 9 */
        for (size_t r = 0; r < g_xs->len; r++) { bus_emit(); if (r >= g_xs->len) break; total += strlen(g_xs->ptr[r].s); }
        g_xs->walkers--;
    }
    printf("total %zu\n", total);
}

/* ---------- A32 a closure held in a variable that another closure writes ---------- */
/* Mode 10, as written:
       name = "first".to_string()
       on_tick = fn(): println("noop")
       fn reset(): on_tick = fn(): println("second")   # writes on_tick: on_tick lives in a box
       on_tick = fn():
           reset()                                      # replaces (and drops) the running closure
           println(name)                                # then uses its own captured state
       on_tick()                                        # Python prints "first"
   The report keeps [CLO-10]'s state for closures in locals ("no call can remove them from under
   themselves"); this one has more than three words of state (name, reset), so it sits in one heap
   block owned by the closure value, dropped when the value is replaced.
   Mode 11, fixed: a closure held where a call may overwrite it (a list element, a field, or a variable
   some closure writes) keeps its state in a counted box, and its body retains the box across a call
   that may release it (the report's rule 3, now applied to this place too). */
typedef struct TickState { uint32_t rc; char *name; size_t len, cap; void *reset_env; } TickState;
typedef struct { void (*fn)(void *st); void *st; } Fn0;
typedef struct { uint32_t rc; Fn0 value; } OnTickBox;          /* the variable on_tick, in its box */
static OnTickBox *g_on_tick; static int g_fixed;
static void noop_fn(void *st) { (void)st; printf("noop\n"); }
static void second_fn(void *st) { (void)st; printf("second\n"); }
static void drop_value(Fn0 v) {
    if (v.fn != noop_fn && v.fn != second_fn && v.st) {
        TickState *s = v.st;
        if (g_fixed) { if (--s->rc) return; }
        free(s->name); free(s);
    }
}
static void reset_fn(void) {                                   /* on_tick = fn(): println("second") */
    Fn0 old = g_on_tick->value;
    g_on_tick->value = (Fn0){ second_fn, NULL };               /* store the new value first, then drop */
    drop_value(old);
}
static void tick_fn(void *stp) {
    TickState *st = stp;
    if (g_fixed) st->rc++;                                     /* used after reset(), which may release it */
    reset_fn();
    printf("%.*s\n", (int)st->len, st->name);
    if (g_fixed) { if (--st->rc == 0) { free(st->name); free(st); } }
}
static void a32_captured_closure(int fixed) {
    g_fixed = fixed;
    g_on_tick = calloc(1, sizeof(OnTickBox)); g_on_tick->rc = 1;
    g_on_tick->value = (Fn0){ noop_fn, NULL };
    TickState *st = calloc(1, sizeof(TickState)); st->rc = 1;
    st->name = _strdup("first"); st->len = 5; st->cap = 6; st->reset_env = g_on_tick;
    Fn0 old = g_on_tick->value; g_on_tick->value = (Fn0){ tick_fn, st }; drop_value(old);
    Fn0 call = g_on_tick->value;                               /* on_tick(): copied out, then called */
    call.fn(call.st);
}

int main(int argc, char **argv) {
    h_start();
    int mode = argc > 1 ? atoi(argv[1]) : 0;
    switch (mode) {
    case 1: a8_assign_slot_first(); break;
    case 2: a8_assign_fixed(); break;
    case 3: a8_swap_rule2(); break;
    case 4: a8_weak(0); break;
    case 5: a8_weak(1); break;
    case 6: a13_narrow(0); break;
    case 7: a13_narrow(1); break;
    case 8: a12_boxed_loop(0); break;
    case 9: a12_boxed_loop(1); break;
    case 10: a32_captured_closure(0); break;
    case 11: a32_captured_closure(1); break;
    default: printf("usage: attacks <1..11>\n");
    }
    return 0;
}
