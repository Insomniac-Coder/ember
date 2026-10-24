/* SALVAGED "copy instead of share" (phase3-design-D1.md), W2: a tree with parent pointers.
   Built from target/redesign/bench/D1/w2.c (class Node, `weak parent`, `children: Array[Node]` as a
   counted copy-on-write buffer, counts on handles copied into a list, the weak read testing the
   target's strong count, bounds checks, swap_remove bumping the shift version). Fixes:
   - FIX 1 (allocation): objects come from a size-class slab (bump, then a free list), not one
     malloc each: nodes made in order sit in order, as plain C's one calloc puts them.
   - FIX 2 (header): a final class with no virtual method and no `cast` needs no type pointer, so
     the header is 8 bytes (strong u32, weak u32), not 16. Node is 32 bytes, as plain C's.
   Built with -DD1_PROVEN: the weak read's liveness test is dropped, as if a whole-program fact
   ("no Node's strong count reaches 0 while main runs": `nodes` holds every node and only grows)
   proved it; an experiment that prices the test.
   Left out, as in every model: [TYP-8] overflow checks; teardown. */
#include "d1rt.h"

typedef struct Node {
    uint32_t strong, weak;     /* header: 8 bytes */
    int32_t value;
    struct Node *parent;       /* weak */
    Buf *children;             /* Array[Node]: strong handles */
} Node;                        /* 32 bytes */
_Static_assert(sizeof(Node) == 32, "Node layout");

/* the size-class slab for 32-byte objects: bump in 64K-object chunks, then a free list */
static char *slab_cur, *slab_end;
static Node *slab_free;
static Node *node_alloc(void) {
    if (slab_free) { Node *n = slab_free; slab_free = (Node *)n->parent; return n; }
    if (slab_cur == slab_end) { size_t sz = (size_t)65536 * sizeof(Node); slab_cur = xmalloc(sz); slab_end = slab_cur + sz; }
    Node *n = (Node *)slab_cur; slab_cur += sizeof(Node); return n;
}
static void node_free(Node *n) { n->parent = slab_free; slab_free = n; }

static void retain_nodes(void *el, uint64_t n) { Node **a = el; for (uint64_t i = 0; i < n; i++) a[i]->strong++; }
static inline void weak_release(Node *p) { if (--p->weak == 0) node_free(p); }
static void node_release(Node *n);
static void node_deinit(Node *n) {
    Buf *c = n->children;
    if (c != &EMPTY && --c->count == 0) {
        for (uint64_t i = 0; i < c->len; i++) node_release(EL(c, Node *)[i]);
        free(c);
    }
    if (n->parent) weak_release(n->parent);
    weak_release(n);
}
static inline void node_release(Node *n) { if (--n->strong == 0) node_deinit(n); }
static inline Node *weak_get(Node **field) {
    Node *p = *field;
#ifndef D1_PROVEN
    if (p && __builtin_expect(p->strong == 0, 0)) { *field = NULL; weak_release(p); p = NULL; }
#endif
    return p;
}
static inline void weak_set(Node **field, Node *v) {
    Node *old = *field;
    if (v) v->weak++;
    *field = v;
    if (old) weak_release(old);
}
static Node *node_new(int32_t value, Node *parent) {
    Node *n = node_alloc();
    n->strong = 1; n->weak = 1;
    n->value = value;
    n->parent = NULL;
    weak_set(&n->parent, parent);
    n->children = &EMPTY;
    return n;
}

int main(void) {
    h_start();
    h_seed(42);
    enum { N = 1000000 };
    Buf *nodes = &EMPTY;
    PUSH(&nodes, Node *, node_new(0, NULL), retain_nodes);
    for (uint32_t i = 1; i < N; i++) {
        uint32_t r = h_rand() % i;
        if (r >= nodes->len) d1_panic("index out of bounds");
        Node *parent = EL(nodes, Node *)[r];
        Node *child = node_new((int32_t)(i % 1000), parent);
        child->strong++;
        PUSH(&parent->children, Node *, child, retain_nodes);
        PUSH(&nodes, Node *, child, retain_nodes);
    }
    H_PHASE("build");
    uint64_t depth_sum = 0;
    for (uint32_t p = 0; p < 10; p++) {
        const Buf *nb = nodes;
        for (uint64_t i = 0; i < nb->len; i++) {
            Node *n = EL(nb, Node *)[i];
            uint64_t d = 0;
            for (;;) {
                Node *up = weak_get(&n->parent);
                if (!up) break;
                d++;
                n = up;
            }
            depth_sum += d;
        }
        for (uint32_t i = 1; i < N; i++) {
            if ((i + p) % 100 != 0) continue;
            uint32_t r = h_rand() % i;
            if (r >= nodes->len) d1_panic("index out of bounds");
            Node *new_parent = EL(nodes, Node *)[r];
            if (i >= nodes->len) d1_panic("index out of bounds");
            Node *child = EL(nodes, Node *)[i];
            Node *old = weak_get(&child->parent);
            if (old) {
                old->strong++;
                const Buf *cb = old->children;
                uint64_t j = 0;
                while (j < cb->len && EL(cb, Node *)[j] != child) j++;
                if (j == cb->len) d1_panic("index_of: not in the list");
                Buf *mb = buf_mut(&old->children, sizeof(Node *), retain_nodes);
                Node *removed = EL(mb, Node *)[j];
                EL(mb, Node *)[j] = EL(mb, Node *)[mb->len - 1];
                mb->len--; mb->shifts++;
                node_release(removed);
                node_release(old);
            }
            weak_set(&child->parent, new_parent);
            child->strong++;
            PUSH(&new_parent->children, Node *, child, retain_nodes);
        }
    }
    H_PHASE("passes");
    uint64_t total = depth_sum;
    for (uint64_t i = 0; i < nodes->len; i++) {
        const Node *n = EL(nodes, Node *)[i];
        total += (uint64_t)((int64_t)n->value * (int64_t)n->children->len);
    }
#ifdef D1_PROVEN
    h_report("d1-w2-proven", total);
#else
    h_report("d1-w2", total);
#endif
    return 0;
}
