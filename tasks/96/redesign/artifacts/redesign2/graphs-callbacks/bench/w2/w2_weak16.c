/* W2, Ember with a `weak` field instead of a back link, written as Python's three-line re-parent:

     class Node:
         value: i32
         parent: weak Node            # reads as Option[Node]; None once the parent has died
         children: Array[Node] = []

     old = node.parent
     if old is not None: old.children.remove(old.children.index_of(node))
     node.parent = new_parent
     new_parent.children.push(node)

   - A weak reference is a plain pointer to the object. An object whose strong count reaches 0 while
     weak references remain is kept as a zombie (fields dropped, block kept) until the weak count is 0,
     as today's [OBJ-3] does. So every read of `parent` checks the target's strong count.
   - The weak count lives in the header: 16 bytes (strong, weak, type index, flags). This is the
     cheapest place for it; with the 8-byte header it would move to a side record, and every weak
     store would look it up there (not modelled: this is a lower bound for that design).
   - Node is 56 bytes. Counts: the removal releases, the push retains (a call lies between them, so
     they are not paired away). */
#include "../ember_model.h"
#define N 1000000

typedef struct Node { uint32_t strong, weak, type, flags; int32_t value; struct Node *parent; EVec children; } Node;
_Static_assert(sizeof(Node) == 56, "Node layout");

static Node *node_new(int32_t value) {
#ifdef SLAB
    static Slab slab;
    Node *n = slab_alloc(&slab, sizeof(Node));
#else
    Node *n = malloc(sizeof(Node));
#endif
    n->strong = 1; n->weak = 1; n->type = 1; n->flags = 0; n->value = value; n->parent = NULL;
    n->children = (EVec){0};
    return n;
}
static inline Node *weak_read(Node *w) { return (w && w->strong != 0) ? w : NULL; }
static inline void weak_store(Node **slot, Node *target) {
    if (target) target->weak++;
    Node *old = *slot;
    if (old && --old->weak == 0) free(old);            /* never reached here */
    *slot = target;
}
static inline void release(Node *n) { if (--n->strong == 0) e_panic("not reached in W2"); }

int main(void) {
    h_start();
    h_seed(42);
    EVec nodes = {0};
    evec_push(&nodes, node_new(0));
    for (uint32_t i = 1; i < N; i++) {
        Node *n = node_new((int32_t)(i % 1000));
        evec_push(&nodes, n);
        size_t r = h_rand() % i;
        e_bounds(r, nodes.len);
        Node *p = nodes.ptr[r];
        weak_store(&n->parent, p);                    /* n.parent = p */
        n->strong++;                                  /* p.children.push(n): retain */
        evec_push(&p->children, n);
    }
    uint64_t depth = 0;
    for (uint32_t p = 0; p < 10; p++) {
        for (size_t j = 0; j < nodes.len; j++)
            for (Node *a = weak_read(((Node *)nodes.ptr[j])->parent); a; a = weak_read(a->parent)) depth++;
        for (uint32_t i = 1; i < N; i++) {
            if ((i + p) % 100 != 0) continue;
            size_t r = h_rand() % i;
            e_bounds(r, nodes.len); e_bounds(i, nodes.len);
            Node *c = nodes.ptr[i], *np = nodes.ptr[r];
            Node *old = weak_read(c->parent);
            if (old) { evec_remove_at(&old->children, evec_index_of(&old->children, c)); release(c); }
            weak_store(&c->parent, np);
            c->strong++;
            evec_push(&np->children, c);
        }
    }
    uint64_t sum = depth;
    for (size_t i = 0; i < nodes.len; i++) {
        Node *n = nodes.ptr[i];
        sum += (uint64_t)((int64_t)n->value * (int64_t)n->children.len);
    }
    #ifdef SLAB
    h_report("w2-ember-weak16-slab", sum);
#else
    h_report("w2-ember-weak16", sum);
#endif
    return 0;
}
