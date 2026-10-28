/* plain_w2v.c (M2, workload "w2v"): run 3's plain W2 (target/redesign2/bench/w2.c, unchanged in
   layout and work) with the one change of this workload: every step of the walk calls a callback,
   which counts the step, through the same two-word slot (code, state) the model reads, `volatile` so
   that the C compiler neither inlines nor devirtualises it. The walk is W2's: it reads `a->parent`
   after the call. A careful C programmer whose callback may free nodes would need protection here; a
   C programmer whose callbacks never free nodes writes exactly this. No checks.
   -DNOVISIT: run 3's plain W2 exactly (depth counted in the walk, no call); checksum 647034814.
   The checksum is W2's either way: the callback counts what the walk counted. */
#include "harness.h"
#define N 1000000
typedef struct Node { int32_t value; uint32_t nch, cap; struct Node *parent; struct Node **ch; } Node;

typedef void (*VisitCode)(void *state, Node *p);
static VisitCode volatile g_visit_code;
static void *volatile g_visit_state;

static void count_visit(void *state, Node *p) { if (p->value >= 0) ++*(uint64_t *)state; }

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
    static uint64_t visits;
    uint64_t depth = 0, sum;
    uint32_t i, p;
    Node *nodes, *n;
    h_start();
    h_seed(42);
    nodes = (Node *)calloc(N, sizeof(Node));
    if (!nodes) return 1;
    for (i = 1; i < N; i++) {
        nodes[i].value = (int32_t)(i % 1000);
        add_child(&nodes[h_rand() % i], &nodes[i]);
    }
    g_visit_code = count_visit; g_visit_state = &visits;
    for (p = 0; p < 10; p++) {
        for (n = nodes; n < nodes + N; n++)
            for (Node *a = n->parent; a; a = a->parent)
#ifdef NOVISIT
                depth++;
#else
                g_visit_code(g_visit_state, a);
#endif
        for (i = 1; i < N; i++) {
            Node *c, *np, *op;
            uint32_t k = 0;
            if ((i + p) % 100 != 0) continue;
            c = &nodes[i]; np = &nodes[h_rand() % i]; op = c->parent;
            while (op->ch[k] != c) k++;
            op->ch[k] = op->ch[--op->nch];
            add_child(np, c);
        }
    }
    sum = depth + visits;
    for (i = 0; i < N; i++) sum += (uint64_t)((int64_t)nodes[i].value * nodes[i].nch);
#ifdef NOVISIT
    h_report("plain-w2v-novisit", sum);
#else
    h_report("plain-w2v", sum);
#endif
    return 0;
}
