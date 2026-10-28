/* plain_tree.c (M2, workload "tree"): what a careful C programmer writes for model_tree.c's work.
   Run 3's plain W2 (target/redesign2/bench/w2.c, copied into tasks/96/redesign/artifacts/
   redesign2/bench/w2.c) turned into rounds: the nodes live in one array allocated once (the array is
   also the index by number), each node's children are a growable array of pointers (first 2, then
   doubling), the parent is a plain pointer, removal is a search and a swap with the last child.
   Each round clears the array, builds, moves a quarter of the nodes, reads every node once, and frees
   every children array (the tree is destroyed; the node array is kept for the next round, as a C
   programmer keeps a pool). No checks, no counts. Same random sequence as the model: one h_rand() per
   built node, one per moved node, in the same order. */
#include "harness.h"
#include <string.h>
#define N 1000000
#define ROUNDS 4
typedef struct Node { int32_t value; uint32_t nch, cap; struct Node *parent; struct Node **ch; } Node;

static void add_child(Node *p, Node *c) {
    if (p->nch == p->cap) {
        p->cap = p->cap ? p->cap * 2 : 2;
        p->ch = (Node **)realloc(p->ch, sizeof(Node *) * p->cap);
        if (!p->ch) exit(1);
    }
    p->ch[p->nch++] = c;
    c->parent = p;
}

int main(void) {
    uint64_t sum = 0;
    uint32_t round, i;
    Node *nodes;
    h_start();
    h_seed(42);
    nodes = (Node *)malloc((size_t)N * sizeof(Node));
    if (!nodes) return 1;
    for (round = 0; round < ROUNDS; round++) {
        memset(nodes, 0, (size_t)N * sizeof(Node));
        for (i = 1; i < N; i++) {
            nodes[i].value = (int32_t)(i % 1000);
            add_child(&nodes[h_rand() % i], &nodes[i]);
        }
        for (i = 1; i < N; i++) {
            Node *c, *np, *op;
            uint32_t k = 0;
            if ((i + round) % 4 != 0) continue;
            c = &nodes[i]; op = c->parent;
            while (op->ch[k] != c) k++;
            op->ch[k] = op->ch[--op->nch];
            np = &nodes[h_rand() % i];
            add_child(np, c);
        }
        for (i = 0; i < N; i++) {
            sum += (uint64_t)((int64_t)nodes[i].value * (int64_t)nodes[i].nch);
            if (nodes[i].parent) sum += (uint64_t)(int64_t)nodes[i].parent->value;
        }
        for (i = 0; i < N; i++) free(nodes[i].ch);
    }
    h_report("plain-tree", sum);
    free(nodes);
    return 0;
}
