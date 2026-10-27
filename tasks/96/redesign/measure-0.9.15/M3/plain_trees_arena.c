/* W1 "trees", plain C with an arena: what a careful C programmer writes for nodes that die together.
   No safety checks. The arena is m3_rt.h's RtArena (the runtime's arena: 1 MB chunks), its bump
   written inline; nothing is freed one by one: the batch ends with one reset, and no walk.
   The work every program of W1 does: 16 batches; each batch builds random expression trees (the
   nodes of a parse: a leaf holds a number, an inner node an operator and two children) until it
   holds at least 2^20 nodes, all alive at once; evaluates every tree; then ends them all.
   A node is 24 bytes: {tag/op, value, left, right}, 24 bytes of arena. Checksum: per tree cs = cs * 1000003 + value, per batch cs += nodes. */
#include "harness.h"
#include "m3_rt.h"

#define NB (1u << 20) /* nodes per batch, at least */
#define BATCHES 16
#define DEPTH 12

typedef struct Node { int32_t op, val; struct Node *l, *r; } Node; /* op 0: a leaf holding val */
typedef char size_check_Node[sizeof(Node) == 24 ? 1 : -1]; /* the layout M3.md states */

static Node *build(RtArena *a, int d, uint64_t *count) {
    uint32_t r = h_rand();
    Node *n;
    if (d == 0 || (r & 3) == 0) {
        n = (Node *)rt_arena_bump(a, sizeof(Node), 8);
        n->op = 0; n->val = (int32_t)((r >> 2) % 100);
    } else {
        Node *l = build(a, d - 1, count);
        Node *rr = build(a, d - 1, count);
        n = (Node *)rt_arena_bump(a, sizeof(Node), 8);
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

int main(void) {
    h_start();
    h_seed(42);
    uint64_t cs = 0;
    RtArena *arena = rt_arena_new((size_t)1 << 20);
    for (int b = 0; b < BATCHES; b++) {
        size_t nroots = 0, cap = 0;
        Node **roots = NULL;
        uint64_t count = 0;
        while (count < NB) {
            if (nroots == cap) { cap = cap ? cap * 2 : 4; roots = (Node **)realloc(roots, cap * sizeof(Node *)); }
            roots[nroots++] = build(arena, DEPTH, &count);
        }
        for (size_t i = 0; i < nroots; i++) cs = cs * 1000003u + eval(roots[i]);
        free(roots);
        rt_arena_reset(arena);
        cs += count;
    }
    rt_arena_free(arena);
    h_report("plain_trees_arena", cs);
    return 0;
}
