/* W2, Ember with a `weak` field implemented as a zeroing weak reference (registration), written as
   Python's three-line re-parent (the same source as w2_weak16.c):

     class Node:
         value: i32
         parent: weak Node            # reads as Option[Node]; None once the parent has died
         children: Array[Node] = []

   - A weak field holds a plain pointer. Reading it is a plain load: no count check, no zombie block.
   - Storing into a weak field registers the field's address with the target: a side record, found
     through a table keyed by the object's address (the 8-byte header has no room for a pointer; the
     header's flag bit says "has a side record" so an object without one pays nothing at death). The
     old target's record forgets the field (a search in its list).
   - An object that dies with a side record clears every registered field to None, then frees its
     block at once (not reached in W2: no node dies).
   - The field's address must not move: `weak` is a field of a class object (objects never move); an
     Array of weak elements re-registers its elements when its buffer moves (not needed here).
   - Lean class: 8-byte header. Counts as in w2_weak16.c (the removal releases, the push retains). */
#include "../ember_model.h"
#define N 1000000

typedef struct Node { uint32_t rc, type; int32_t value; struct Node *parent; EVec children; } Node;
_Static_assert(sizeof(Node) == 48, "Node layout");
#define HAS_SIDE 0x80000000u

/* the side table: object address -> list of registered weak fields (open addressing) */
typedef struct { Node *key; EVec fields; } Side;
static Side *tab; static size_t tab_cap, tab_len;
static inline size_t hash_ptr(const void *p) { uint64_t x = (uint64_t)(uintptr_t)p; x ^= x >> 33; x *= 0xff51afd7ed558ccdull; x ^= x >> 33; return (size_t)x; }
static Side *side_find(Node *o, int create) {
    if (create && (tab_len + 1) * 2 > tab_cap) {
        size_t oc = tab_cap; Side *ot = tab;
        tab_cap = oc ? oc * 2 : 1024; tab = calloc(tab_cap, sizeof(Side)); tab_len = 0;
        for (size_t i = 0; i < oc; i++) if (ot[i].key) {
            size_t j = hash_ptr(ot[i].key) & (tab_cap - 1);
            while (tab[j].key) j = (j + 1) & (tab_cap - 1);
            tab[j] = ot[i]; tab_len++;
        }
        free(ot);
    }
    size_t j = hash_ptr(o) & (tab_cap - 1);
    while (tab[j].key) { if (tab[j].key == o) return &tab[j]; j = (j + 1) & (tab_cap - 1); }
    if (!create) return NULL;
    tab[j].key = o; tab[j].fields = (EVec){0}; tab_len++;
    o->rc |= HAS_SIDE;
    return &tab[j];
}
static inline void weak_store(Node **field, Node *target) {
    Node *old = *field;
    if (old) { Side *s = side_find(old, 0); evec_remove_at(&s->fields, evec_index_of(&s->fields, field)); }
    if (target) evec_push(&side_find(target, 1)->fields, field);
    *field = target;
}

static Node *node_new(int32_t value) {
#ifdef SLAB
    static Slab slab;
    Node *n = slab_alloc(&slab, sizeof(Node));
#else
    Node *n = malloc(sizeof(Node));
#endif
    n->rc = 1; n->type = 1; n->value = value; n->parent = NULL;
    n->children = (EVec){0};
    return n;
}
static inline void release(Node *n) { if ((--n->rc & ~HAS_SIDE) == 0) e_panic("not reached in W2"); }

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
        weak_store(&n->parent, p);
        n->rc++;
        evec_push(&p->children, n);
    }
    uint64_t depth = 0;
    for (uint32_t p = 0; p < 10; p++) {
        for (size_t j = 0; j < nodes.len; j++)
            for (Node *a = ((Node *)nodes.ptr[j])->parent; a; a = a->parent) depth++;
        for (uint32_t i = 1; i < N; i++) {
            if ((i + p) % 100 != 0) continue;
            size_t r = h_rand() % i;
            e_bounds(r, nodes.len); e_bounds(i, nodes.len);
            Node *c = nodes.ptr[i], *np = nodes.ptr[r];
            Node *old = c->parent;
            if (old) { evec_remove_at(&old->children, evec_index_of(&old->children, c)); release(c); }
            weak_store(&c->parent, np);
            c->rc++;
            evec_push(&np->children, c);
        }
    }
    uint64_t sum = depth;
    for (size_t i = 0; i < nodes.len; i++) {
        Node *n = nodes.ptr[i];
        sum += (uint64_t)((int64_t)n->value * (int64_t)n->children.len);
    }
#ifdef SLAB
    h_report("w2-ember-zweak-slab", sum);
#else
    h_report("w2-ember-zweak", sum);
#endif
    return 0;
}
