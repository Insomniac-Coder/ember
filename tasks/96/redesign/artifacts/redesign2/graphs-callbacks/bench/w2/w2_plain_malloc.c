/* W2, plain C written the way a class-based program is laid out: one allocation per node, a list of
   node pointers, order-keeping removal from the old parent. No header, no counts, no checks. This is
   the "same work, same way" baseline for the Ember models.
     -DSLAB      nodes come from the per-class slab of ember_model.h instead of one malloc each
     -DARRAY24   the child list is Ember's Array layout ({ptr, 64-bit len, 64-bit cap}, first growth
                 to 4) instead of {32-bit count, 32-bit capacity, ptr} growing from 2: the node is
                 40 bytes instead of 32. It takes apart what the Array layout costs. */
#include "../ember_model.h"
#define N 1000000

#ifdef ARRAY24
typedef struct Node { int32_t value; struct Node *parent; EVec ch; } Node;
_Static_assert(sizeof(Node) == 40, "Node layout");
#define NCH(n) ((n)->ch.len)
#define CHI(n, k) ((n)->ch.ptr[k])
static void add_child(Node *p, Node *c) { evec_push(&p->ch, c); c->parent = p; }
static void remove_child(Node *op, Node *c) {
    size_t k = 0;
    while (op->ch.ptr[k] != c) k++;
    memmove(&op->ch.ptr[k], &op->ch.ptr[k + 1], sizeof(void *) * (op->ch.len - k - 1));
    op->ch.len--;
}
#else
typedef struct Node { int32_t value; uint32_t nch, cap; struct Node *parent; struct Node **ch; } Node;
_Static_assert(sizeof(Node) == 32, "Node layout");
#define NCH(n) ((n)->nch)
static void add_child(Node *p, Node *c) {
    if (p->nch == p->cap) { p->cap = p->cap ? p->cap * 2 : 2; p->ch = realloc(p->ch, sizeof(Node *) * p->cap); }
    p->ch[p->nch++] = c;
    c->parent = p;
}
static void remove_child(Node *op, Node *c) {
    uint32_t k = 0;
    while (op->ch[k] != c) k++;
    memmove(&op->ch[k], &op->ch[k + 1], sizeof(Node *) * (op->nch - k - 1));
    op->nch--;
}
#endif

static Node *node_new(int32_t value) {
#ifdef SLAB
    static Slab slab;
    Node *n = slab_alloc(&slab, sizeof(Node));
    memset(n, 0, sizeof(Node));
#else
    Node *n = calloc(1, sizeof(Node));
#endif
    n->value = value;
    return n;
}

int main(void) {
    h_start();
    h_seed(42);
    Node **nodes = malloc(sizeof(Node *) * N);
    nodes[0] = node_new(0);
    for (uint32_t i = 1; i < N; i++) {
        Node *n = node_new((int32_t)(i % 1000));
        nodes[i] = n;
        add_child(nodes[h_rand() % i], n);
    }
    uint64_t depth = 0;
    for (uint32_t p = 0; p < 10; p++) {
        for (uint32_t j = 0; j < N; j++)
            for (Node *a = nodes[j]->parent; a; a = a->parent) depth++;
        for (uint32_t i = 1; i < N; i++) {
            if ((i + p) % 100 != 0) continue;
            Node *c = nodes[i], *np = nodes[h_rand() % i];
            remove_child(c->parent, c);
            add_child(np, c);
        }
    }
    uint64_t sum = depth;
    for (uint32_t i = 0; i < N; i++) sum += (uint64_t)((int64_t)nodes[i]->value * (int64_t)NCH(nodes[i]));
#if defined(SLAB) && defined(ARRAY24)
    h_report("w2-plain-slab-array24", sum);
#elif defined(SLAB)
    h_report("w2-plain-slab", sum);
#elif defined(ARRAY24)
    h_report("w2-plain-malloc-array24", sum);
#else
    h_report("w2-plain-malloc", sum);
#endif
    return 0;
}
