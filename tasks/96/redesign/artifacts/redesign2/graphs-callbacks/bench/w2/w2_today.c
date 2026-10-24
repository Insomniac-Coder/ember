/* W2, Ember today, as the spec lays it out ([OBJ-1], [EXC-19], [WK-12], [RC-1]):

     class Node:
         value: i32
         parent: Weak[Node] = Weak.empty()
         children: Array[Node] = []

     d = n
     while True:
         match d.parent.upgrade():
             Some(p): depth_sum += 1; d = p
             None: break

   - 24-byte header (strong, weak, access, flags, type pointer); `children` is the one non-Copy field,
     so one access word plus a zero word (8 bytes). Node is 72 bytes.
   - `upgrade()` checks the strong count and the deinitialising flag and returns a retained handle;
     `d = p` releases the old one. The loop's first `d = n` copies a handle: a retain.
   - The re-parent is today's spelling (probes/a8_reparent_python.em): upgrade the old parent, search
     and remove (a release), store a new Weak (weak counts), push (a retain). A method call on the
     `children` field is a write access on its access word ([EXC-19]): begin, check, end. */
#include "../ember_model.h"
#define N 1000000

typedef struct Node {
    uint32_t strong, weak, access, flags; const void *type;
    uint32_t access_children, access_pad;
    int32_t value; struct Node *parent; EVec children;
} Node;
_Static_assert(sizeof(Node) == 72, "Node layout");
static const int node_type_info = 1;

static Node *node_new(int32_t value) {
#ifdef SLAB
    static Slab slab;
    Node *n = slab_alloc(&slab, sizeof(Node));
#else
    Node *n = malloc(sizeof(Node));
#endif
    n->strong = 1; n->weak = 1; n->access = 0; n->flags = 0; n->type = &node_type_info;
    n->access_children = 0; n->access_pad = 0;
    n->value = value; n->parent = NULL; n->children = (EVec){0};
    return n;
}
static inline void retain(Node *n) { n->strong++; }
static inline void release(Node *n) { if (--n->strong == 0) e_panic("not reached in W2"); }
static inline Node *upgrade(Node *w) {
    if (w && w->strong != 0 && !(w->flags & 1)) { w->strong++; return w; }
    return NULL;
}
static inline void weak_assign(Node **slot, Node *target) {
    if (target) target->weak++;
    Node *old = *slot;
    if (old && --old->weak == 0) free(old);
    *slot = target;
}
static inline void begin_write(uint32_t *w) { if (*w != 0) e_panic("exclusivity"); *w = 0x80000000u; }
static inline void end_write(uint32_t *w) { *w = 0; }

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
        weak_assign(&n->parent, p);
        retain(n);
        begin_write(&p->access_children); evec_push(&p->children, n); end_write(&p->access_children);
    }
    uint64_t depth = 0;
    for (uint32_t p = 0; p < 10; p++) {
        for (size_t j = 0; j < nodes.len; j++) {
            Node *d = nodes.ptr[j];
            retain(d);
            for (;;) {
                Node *up = upgrade(d->parent);
                if (!up) break;
                depth++;
                release(d);
                d = up;
            }
            release(d);
        }
        for (uint32_t i = 1; i < N; i++) {
            if ((i + p) % 100 != 0) continue;
            size_t r = h_rand() % i;
            e_bounds(i, nodes.len);
            Node *c = nodes.ptr[i];
            Node *old = upgrade(c->parent);
            if (old) {
                begin_write(&old->access_children);
                size_t k = evec_index_of(&old->children, c);
                end_write(&old->access_children);
                begin_write(&old->access_children);
                evec_remove_at(&old->children, k);
                end_write(&old->access_children);
                release(c);
                release(old);
            }
            e_bounds(r, nodes.len);
            Node *np = nodes.ptr[r];
            weak_assign(&c->parent, np);
            retain(c);
            begin_write(&np->access_children); evec_push(&np->children, c); end_write(&np->access_children);
        }
    }
    uint64_t sum = depth;
    for (size_t i = 0; i < nodes.len; i++) {
        Node *n = nodes.ptr[i];
        sum += (uint64_t)((int64_t)n->value * (int64_t)n->children.len);
    }
    #ifdef SLAB
    h_report("w2-ember-today-slab", sum);
#else
    h_report("w2-ember-today", sum);
#endif
    return 0;
}
