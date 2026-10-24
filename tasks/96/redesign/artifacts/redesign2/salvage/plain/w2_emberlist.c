/* VARIANT of the plain C W2 baseline with Ember's List layout instead of the C programmer's:
   each node's children are {pointer, int64 len, int64 cap} growing by doubling from 4 ([HEAP-2]),
   where plain/w2.c keeps uint32 count and capacity and starts at 2. Everything else is plain/w2.c.
   It splits the models' W2 numbers into what the list layout costs (this file against w2.c) and
   what the memory model costs (a model against this file). No safety checks. */
#include "../sh.h"
#define N 1000000
typedef struct Node { int32_t value; struct Node *parent; struct Node **ch; int64_t nch, cap; } Node; /* 40 bytes */

static void add_child(Node *p, Node *c) {
    if (p->nch == p->cap) { p->cap = p->cap ? p->cap * 2 : 4; p->ch = realloc(p->ch, sizeof(Node *) * (size_t)p->cap); }
    p->ch[p->nch++] = c;
    c->parent = p;
}

int main(void) {
    h_start();
    h_seed(42);
    Node *nodes = calloc(N, sizeof(Node));
    for (uint32_t i = 1; i < N; i++) {
        nodes[i].value = (int32_t)(i % 1000);
        add_child(&nodes[h_rand() % i], &nodes[i]);
    }
    H_PHASE("build");
    uint64_t depth = 0;
    for (uint32_t p = 0; p < 10; p++) {
        for (Node *n = nodes; n < nodes + N; n++)
            for (Node *a = n->parent; a; a = a->parent) depth++;
        for (uint32_t i = 1; i < N; i++) {
            if ((i + p) % 100 != 0) continue;
            Node *c = &nodes[i], *np = &nodes[h_rand() % i], *op = c->parent;
            int64_t k = 0;
            while (op->ch[k] != c) k++;
            op->ch[k] = op->ch[--op->nch];
            add_child(np, c);
        }
    }
    H_PHASE("passes");
    uint64_t sum = depth;
    for (uint32_t i = 0; i < N; i++) sum += (uint64_t)((int64_t)nodes[i].value * nodes[i].nch);
    h_report("plain-w2-emberlist", sum);
    return 0;
}
