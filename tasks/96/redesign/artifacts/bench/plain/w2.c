/* Plain C baseline, W2 (tree with parent pointers). No safety checks. All nodes in one
   allocation; each node's children are a growable array of node pointers; removal from the
   old parent is a linear search and a swap with the last child. */
#include "../harness.h"
#define N 1000000
typedef struct Node { int32_t value; uint32_t nch, cap; struct Node *parent; struct Node **ch; } Node;

static void add_child(Node *p, Node *c) {
    if (p->nch == p->cap) { p->cap = p->cap ? p->cap * 2 : 2; p->ch = realloc(p->ch, sizeof(Node *) * p->cap); }
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
    uint64_t depth = 0;
    for (uint32_t p = 0; p < 10; p++) {
        for (Node *n = nodes; n < nodes + N; n++)
            for (Node *a = n->parent; a; a = a->parent) depth++;
        for (uint32_t i = 1; i < N; i++) {
            if ((i + p) % 100 != 0) continue;
            Node *c = &nodes[i], *np = &nodes[h_rand() % i], *op = c->parent;
            uint32_t k = 0;
            while (op->ch[k] != c) k++;
            op->ch[k] = op->ch[--op->nch];
            add_child(np, c);
        }
    }
    uint64_t sum = depth;
    for (uint32_t i = 0; i < N; i++) sum += (uint64_t)((int64_t)nodes[i].value * nodes[i].nch);
    h_report("plain-w2", sum);
    return 0;
}
