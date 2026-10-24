/* w2v_backlink.c: the graphs group's w2_backlink.c (copied unchanged otherwise), with one change to
   the walk: every step calls a visitor the compiler cannot see (a function value), as a walk that
   reports each ancestor does:

       fn walk(nodes: Array[Node], visit: fn(Node)) -> int:
           depth = 0
           for n in nodes:
               a = n.parent
               while a is not None:
                   visit(a)                 # a function value: all effects ([EFF-2]); may drop a Node
                   depth += 1
                   a = a.parent             # `a` is used after the call: retained across it

     -DHAZARD   a cheaper protection for the same purpose (the save tried here): before the call the
                walk pushes `a` on the thread's stack of protected objects (one store, no write to the
                node); a release that drops a count to zero while its object is on that stack defers
                the whole drop (fields and block) to the moment the stack entry is popped; after the
                call the walk pops and tests one word for deferred drops. Objects stay on their
                thread (the threads group's rule for non-atomic counts), so only this thread's stack
                is checked. The stack is a global here (a thread-local in a real runtime).
     -DRETAIN   the ownership group's rule (phase3b-ownership.md 2.3: a handle read from a place is not
                retained only when nothing in its life may drop an object of its class): one count pair
                per step. Without it: the same walk when the visitor's effects are known.

   The original comment follows.

   W2, Ember with a back link (the proposal), as its compiler would emit this program:

     class Node:
         value: i32
         parent: Option[Node] = backlink(children)
         children: Array[Node] = []

     nodes[h_rand() as int % i].children.push(nodes[i])     # re-parent: one line

   - The lean class (phase3b-ownership.md 2.3): an 8-byte header, a 32-bit count and a 32-bit type
     index. Node is 48 bytes: header 8, value 4 (+4), parent 8, children 24.
   - `parent` is a plain pointer: no count, no check, no weak count. The language keeps it equal to
     the node whose `children` holds this one: a push into `children` first takes the node out of its
     old holder's `children` (search and order-keeping remove), then sets `parent`. A holder that dies
     clears the `parent` of each child before releasing it (not reached here: no node dies).
   - Counts: a new node is held by `nodes` and by its parent's `children` (count 2, set once). A
     re-parent moves the handle from the old `children` to the new one: no count change.
   - Bounds checks on `nodes[...]`, as Ember indexes. The search runs inside the old holder's length. */
#include "ember_model.h"
#define N 1000000

#ifdef NOHEADER   /* decomposition only: no count, no type index (not safe for shared objects) */
typedef struct Node { int32_t value; struct Node *parent; EVec children; } Node;
_Static_assert(sizeof(Node) == 40, "Node layout");
#define SET_RC(n, v) ((void)0)
#else
typedef struct Node { uint32_t rc, type; int32_t value; struct Node *parent; EVec children; } Node;
_Static_assert(sizeof(Node) == 48, "Node layout");
#define SET_RC(n, v) ((n)->rc = (v))
#endif

static uint64_t g_visits;
static void visit_count(Node *a) { g_visits += (uint64_t)(a->value & 1); }
static void (*volatile g_visit)(Node *) = visit_count;   /* chosen at run time: opaque to the C compiler */
static Node *g_hz[64]; static size_t g_hztop, g_deferred;   /* protected objects; deferred drops */
static void run_deferred(void) { e_panic("not reached: no node dies in W2"); }
static inline void node_release(Node *n) {
#ifndef NOHEADER
    if (--n->rc == 0) e_panic("not reached: every node is held by `nodes`");
#endif
}

static Node *node_new(int32_t value) {
#ifdef SLAB
    static Slab slab;
    Node *n = slab_alloc(&slab, sizeof(Node));
#else
    Node *n = malloc(sizeof(Node));
#endif
    SET_RC(n, 1);
#ifndef NOHEADER
    n->type = 1;
#endif
    n->value = value; n->parent = NULL;
    n->children = (EVec){0};
    return n;
}

/* holder.children.push(c), with c.parent kept by the language */
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
        evec_push(&nodes, n);                         /* moves n into nodes */
        size_t r = h_rand() % i;
        e_bounds(r, nodes.len);
        SET_RC(n, 2);                                 /* a second owning place: retain */
        children_push((Node *)nodes.ptr[r], n);
    }
    uint64_t depth = 0;
    for (uint32_t p = 0; p < 10; p++) {
        for (size_t j = 0; j < nodes.len; j++)
            for (Node *a = ((Node *)nodes.ptr[j])->parent; a; ) {
#if defined(RETAIN)
                a->rc++;                              /* retain: the call may drop a Node */
#elif defined(HAZARD)
                g_hz[g_hztop++] = a;                  /* protect: on this thread's stack */
#endif
                void (*visit)(Node *) = g_visit;
                visit(a);
                depth++;
                Node *next = a->parent;
#if defined(RETAIN)
                node_release(a);
#elif defined(HAZARD)
                g_hztop--;
                if (g_deferred) run_deferred();
#endif
                a = next;
            }
        for (uint32_t i = 1; i < N; i++) {
            if ((i + p) % 100 != 0) continue;
            size_t r = h_rand() % i;
            e_bounds(r, nodes.len); e_bounds(i, nodes.len);
            children_push((Node *)nodes.ptr[r], (Node *)nodes.ptr[i]);
        }
    }
    uint64_t sum = depth + g_visits;
    for (size_t i = 0; i < nodes.len; i++) {
        Node *n = nodes.ptr[i];
        sum += (uint64_t)((int64_t)n->value * (int64_t)n->children.len);
    }
#if defined(RETAIN)
    h_report("w2v-backlink-retain", sum);
#elif defined(HAZARD)
    h_report("w2v-backlink-hazard", sum);
#else
    h_report("w2v-backlink", sum);
#endif
    return 0;
}
