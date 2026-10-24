/* W2 with back links, taken apart: graphs-callbacks/bench/w2/w2_backlink.c (always -DSLAB) with four
   switches, so the gap between the back-link model and plain C in the same layout can be put on
   its causes:
     -DNOHEADER    no 8-byte header (as in the original; not safe for shared objects)
     -DPLAINNODES  the `nodes` list is plain C's: one malloc of N pointers, no bounds checks on
                   nodes[...] (the original grows it by push from 4 and checks every index, as Ember does)
     -DPLAINCHILD  the search and remove in children_push are plain C's (no "not found" panic, no
                   bounds check on the found index)
     -DHDR4        a 4-byte header (the count only; a class that is not `open` and has no base needs no
                   type index), which packs with `value: i32`: the node is 40 bytes, as without a header
   With all three, the program is plain C's w2_plain_malloc.c -DSLAB -DARRAY24 plus one `if (old)`.
   What stays in every variant is the back link itself: c->parent read, old holder searched, new
   holder pushed, c->parent stored. */
#include "../../graphs-callbacks/bench/ember_model.h"
#define N 1000000

#ifdef NOHEADER
typedef struct Node { int32_t value; struct Node *parent; EVec children; } Node;
#define SET_RC(n, v) ((void)0)
#elif defined(HDR4)   /* a final class with no base: a 4-byte header (count only), packed with value */
typedef struct Node { uint32_t rc; int32_t value; struct Node *parent; EVec children; } Node;
_Static_assert(sizeof(Node) == 40, "Node layout");
#define SET_RC(n, v) ((n)->rc = (v))
#else
typedef struct Node { uint32_t rc, type; int32_t value; struct Node *parent; EVec children; } Node;
#define SET_RC(n, v) ((n)->rc = (v))
#endif

static Node *node_new(int32_t value) {
    static Slab slab;
    Node *n = slab_alloc(&slab, sizeof(Node));
    SET_RC(n, 1);
#if !defined(NOHEADER) && !defined(HDR4)
    n->type = 1;
#endif
    n->value = value; n->parent = NULL;
    n->children = (EVec){0};
    return n;
}

static inline void children_push(Node *holder, Node *c) {
    Node *old = c->parent;
    if (old) {
#ifdef PLAINCHILD
        size_t k = 0;
        while (old->children.ptr[k] != c) k++;
        memmove(&old->children.ptr[k], &old->children.ptr[k + 1], sizeof(void *) * (old->children.len - k - 1));
        old->children.len--;
#else
        evec_remove_at(&old->children, evec_index_of(&old->children, c));
#endif
    }
    evec_push(&holder->children, c);
    c->parent = holder;
}

#ifdef PLAINNODES
#define NODE(i) (nodes[i])
#define NLEN N
#define CHECK(i) ((void)0)
#else
#define NODE(i) ((Node *)nodes.ptr[i])
#define NLEN nodes.len
#define CHECK(i) e_bounds((i), nodes.len)
#endif

int main(void) {
    h_start();
    h_seed(42);
#ifdef PLAINNODES
    Node **nodes = malloc(sizeof(Node *) * N);
    nodes[0] = node_new(0);
#else
    EVec nodes = {0};
    evec_push(&nodes, node_new(0));
#endif
    for (uint32_t i = 1; i < N; i++) {
        Node *n = node_new((int32_t)(i % 1000));
#ifdef PLAINNODES
        nodes[i] = n;
#else
        evec_push(&nodes, n);
#endif
        size_t r = h_rand() % i;
        CHECK(r);
        SET_RC(n, 2);
        children_push(NODE(r), n);
    }
    uint64_t depth = 0;
    for (uint32_t p = 0; p < 10; p++) {
        for (size_t j = 0; j < NLEN; j++)
            for (Node *a = NODE(j)->parent; a; a = a->parent) depth++;
        for (uint32_t i = 1; i < N; i++) {
            if ((i + p) % 100 != 0) continue;
            size_t r = h_rand() % i;
            CHECK(r); CHECK(i);
            children_push(NODE(r), NODE(i));
        }
    }
    uint64_t sum = depth;
    for (size_t i = 0; i < NLEN; i++) {
        Node *n = NODE(i);
        sum += (uint64_t)((int64_t)n->value * (int64_t)n->children.len);
    }
    char name[64];
    snprintf(name, sizeof name, "w2-backlink%s%s%s",
#ifdef NOHEADER
             "-nohdr",
#elif defined(HDR4)
             "-hdr4",
#else
             "",
#endif
#ifdef PLAINNODES
             "-plainnodes",
#else
             "",
#endif
#ifdef PLAINCHILD
             "-plainchild"
#else
             ""
#endif
    );
    h_report(name, sum);
    return 0;
}
