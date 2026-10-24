/* combined-w2: W2 (tree with parent pointers) as the combined memory model's compiler would emit it.
   Report: tasks/96/redesign/phase3b-combined.md, section 6.2. Built from the graphs group's
   target/redesign2/graphs-callbacks/bench/w2/w2_backlink.c (-DSLAB) with the header its attacker
   proposed (phase3b-graphs-callbacks-attack.md 3.6, -DHDR4).

   The Ember program (the model's syntax):

       class Node:
           value: i32
           parent: Option[Node] = backlink(children)    # kept equal to "the node whose children hold me"
           children: Array[Node] = []

       nodes = [Node(value=0)]
       for i in range(1, N):
           n = Node(value=i % 1000)
           nodes.push(n)
           nodes[rand() % i].children.push(n)           # also sets n.parent
       for p in range(10):
           for n in nodes:
               m = n.parent
               while m is not None:
                   depth_sum += 1
                   m = m.parent
           for i in range(1, N):
               if (i + p) % 100 == 0:
                   nodes[rand() % i].children.push(nodes[i])   # takes it out of its old parent first

   What the model emits, and nothing else:
   - Node is a class that is not `open` and has no base class, so no handle to it ever needs its
     type at run time (a `dyn` value carries its table beside the pointer): the header is the 4-byte
     count alone, which packs with the 4-byte `value`. Node is 40 bytes, plain C's same-layout size.
   - Class objects come from a slab for their size (1 MiB chunks, carved in order; a free list per
     size, not reached here: no node dies).
   - `parent` is a plain pointer: no count, no check. A push into `children` first takes the node out
     of its old holder's `children` (search, then the order-keeping remove of Python's list.remove),
     then sets `parent`. The handle moves from one list to the other: no count change.
   - Counts: a new node is stored in `nodes` and in its parent's `children`: one retain (count 2).
     Reading handles in the walk (`m = m.parent`) changes no count: nothing in the walk can drop a
     Node (field effects).
   - Bounds checks on `nodes[...]` (the compiler cannot prove len(nodes) == i); the search runs inside
     the old holder's length; remove_at checks its position.
   - Ember's Array {ptr, len, cap}, 64-bit len and cap, first growth to 4 (helpers below).
   - Integer overflow checks left out, as in every model.
   Self-check build: -DLINK_CHECK checks every back link and count at the end. */
#include "../../../redesign/bench/harness.h"   /* path changed when the file moved into first-run/ */
#include <string.h>
#define N 1000000

/* Ember's runtime pieces this program uses (as target/redesign2/graphs-callbacks/bench/ember_model.h):
   an Array of handles {ptr, len, cap}, first growth to 4 then doubling; a checked index panics. */
typedef struct { void **ptr; size_t len, cap; } EVec;
static __declspec(noreturn) void e_panic(const char *what) { fprintf(stderr, "panic: %s\n", what); exit(101); }
static inline void e_bounds(size_t i, size_t n) { if (__builtin_expect(i >= n, 0)) e_panic("index out of range"); }
static void evec_grow(EVec *v) {
    size_t cap = v->cap < 2 ? 4 : v->cap * 2;
    v->ptr = realloc(v->ptr, cap * sizeof(void *));
    if (!v->ptr) e_panic("out of memory");
    v->cap = cap;
}
static inline void evec_push(EVec *v, void *x) { if (v->len == v->cap) evec_grow(v); v->ptr[v->len++] = x; }
static inline void evec_remove_at(EVec *v, size_t k) {        /* order-keeping, as Python's list.remove */
    e_bounds(k, v->len);
    memmove(&v->ptr[k], &v->ptr[k + 1], sizeof(void *) * (v->len - k - 1));
    v->len--;
}
static inline size_t evec_index_of(EVec *v, void *x) {
    for (size_t k = 0; k < v->len; k++) if (v->ptr[k] == x) return k;
    e_panic("value not in list");
}
/* A slab per object size: 1 MiB chunks carved in order, no per-object allocator header. */
typedef struct { char *cur, *end; } Slab;
static inline void *slab_alloc(Slab *s, size_t size) {
    if ((size_t)(s->end - s->cur) < size) {
        s->cur = malloc(1u << 20);
        if (!s->cur) e_panic("out of memory");
        s->end = s->cur + (1u << 20);
    }
    void *p = s->cur; s->cur += size; return p;
}

typedef struct Node { uint32_t rc; int32_t value; struct Node *parent; EVec children; } Node;
_Static_assert(sizeof(Node) == 40, "Node layout");

static Slab node_slab;
static Node *node_new(int32_t value) {
    Node *n = slab_alloc(&node_slab, sizeof(Node));
    n->rc = 1; n->value = value; n->parent = NULL;
    n->children = (EVec){0};
    return n;
}
static inline void retain(Node *n) {
    if (__builtin_expect(n->rc - 1u >= 0x0FFFFFFEu, 0)) e_panic("count past 28 bits: moves to a side record (not reached here)");
    n->rc++;
}
/* holder.children.push(c), with c.parent kept by the language (back-link rules 2 and 3) */
static inline void children_push(Node *holder, Node *c) {
    Node *old = c->parent;
    if (old) evec_remove_at(&old->children, evec_index_of(&old->children, c));
    evec_push(&holder->children, c);
    c->parent = holder;
}

int main(void) {
    h_start();
    h_seed(42);
    EVec nodes = {0};
    evec_push(&nodes, node_new(0));
    for (uint32_t i = 1; i < N; i++) {
        Node *n = node_new((int32_t)(i % 1000));
        evec_push(&nodes, n);                         /* n moves into nodes; the name n now names nodes[i] */
        size_t r = h_rand() % i;
        e_bounds(r, nodes.len);
        retain(n);                                    /* a second owning place */
        children_push((Node *)nodes.ptr[r], n);
    }
    uint64_t depth = 0;
    for (uint32_t p = 0; p < 10; p++) {
        for (size_t j = 0; j < nodes.len; j++)
            for (Node *m = ((Node *)nodes.ptr[j])->parent; m; m = m->parent) depth++;
        for (uint32_t i = 1; i < N; i++) {
            if ((i + p) % 100 != 0) continue;
            size_t r = h_rand() % i;
            e_bounds(r, nodes.len); e_bounds(i, nodes.len);
            children_push((Node *)nodes.ptr[r], (Node *)nodes.ptr[i]);
        }
    }
    uint64_t sum = depth;
    for (size_t i = 0; i < nodes.len; i++) {
        Node *n = nodes.ptr[i];
        sum += (uint64_t)((int64_t)n->value * (int64_t)n->children.len);
    }
#ifdef LINK_CHECK   /* self-check build: every back link names the node whose children hold it */
    size_t held = 0;
    for (size_t i = 0; i < nodes.len; i++) {
        Node *n = nodes.ptr[i];
        for (size_t c = 0; c < n->children.len; c++) {
            Node *ch = n->children.ptr[c];
            if (ch->parent != n) e_panic("link check: a child's parent is not its holder");
            if (ch->rc != 2) e_panic("link check: a child's count is not 2");
            held++;
        }
        if (i > 0 && (!n->parent || evec_index_of(&n->parent->children, n) >= n->parent->children.len))
            e_panic("link check: a node is not in its parent's children");
    }
    if (held != nodes.len - 1) e_panic("link check: a node is held twice or not at all");
    puts("link check ok");
    (void)held;
#endif
    h_report("combined-w2", sum);
    return 0;
}
