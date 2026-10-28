/* W1 "trees", plain C with malloc/free: the C programmer who has no arena. No safety checks.
   The work every program of W1 does: 16 batches; each batch builds random expression trees (the
   nodes of a parse: a leaf holds a number, an inner node an operator and two children) until it
   holds at least 2^20 nodes, all alive at once; evaluates every tree; then ends them all.
   A node is 24 bytes: {tag/op, value, left, right}. Each node is one malloc, each end one free
   (a post-order walk). Checksum: per tree cs = cs * 1000003 + value, per batch cs += nodes. */
#include "harness.h"

#define NB (1u << 20) /* nodes per batch, at least */
#define BATCHES 16
#define DEPTH 12

typedef struct Node { int32_t op, val; struct Node *l, *r; } Node; /* op 0: a leaf holding val */
typedef char size_check_Node[sizeof(Node) == 24 ? 1 : -1]; /* the layout M3.md states */

static Node *build(int d, uint64_t *count) {
    uint32_t r = h_rand();
    Node *n;
    if (d == 0 || (r & 3) == 0) {
        n = (Node *)malloc(sizeof(Node));
        n->op = 0; n->val = (int32_t)((r >> 2) % 100);
    } else {
        Node *l = build(d - 1, count);
        Node *rr = build(d - 1, count);
        n = (Node *)malloc(sizeof(Node));
        n->op = (int32_t)(1 + (r >> 2) % 3); n->val = 0; n->l = l; n->r = rr;
    }
    ++*count;
    return n;
}

static uint32_t eval(const Node *n) {
    switch (n->op) {
    case 0: return (uint32_t)n->val;
    case 1: return eval(n->l) + eval(n->r);
    case 2: return eval(n->l) * eval(n->r);
    default: return eval(n->l) - eval(n->r);
    }
}

static void end_tree(Node *n) {
    if (n->op != 0) { end_tree(n->l); end_tree(n->r); }
    free(n);
}

int main(void) {
    h_start();
    h_seed(42);
    uint64_t cs = 0;
    for (int b = 0; b < BATCHES; b++) {
        size_t nroots = 0, cap = 0;
        Node **roots = NULL;
        uint64_t count = 0;
        while (count < NB) {
            if (nroots == cap) { cap = cap ? cap * 2 : 4; roots = (Node **)realloc(roots, cap * sizeof(Node *)); }
            roots[nroots++] = build(DEPTH, &count);
        }
        for (size_t i = 0; i < nroots; i++) cs = cs * 1000003u + eval(roots[i]);
        for (size_t i = 0; i < nroots; i++) end_tree(roots[i]);
        free(roots);
        cs += count;
    }
    h_report("plain_trees_malloc", cs);
    return 0;
}
