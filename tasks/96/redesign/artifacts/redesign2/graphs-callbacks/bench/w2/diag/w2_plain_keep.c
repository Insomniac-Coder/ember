/* W2 (tree with parent pointers), plain C, all nodes in one calloc, as phase 3's plain/w2.c, but the
   re-parent removes the node from its old parent's child array keeping the order (memmove), as an
   Array's remove does. No safety checks. */
#include "harness.h"
#include <string.h>
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
    LARGE_INTEGER qa, qf; QueryPerformanceFrequency(&qf); QueryPerformanceCounter(&qa);
    double t_build = (double)(qa.QuadPart - h_t0.QuadPart) * 1000.0 / qf.QuadPart, t_walk = 0, t_rep = 0;
    uint64_t depth = 0;
    for (uint32_t p = 0; p < 10; p++) {
        { LARGE_INTEGER s; QueryPerformanceCounter(&s);
        for (Node *n = nodes; n < nodes + N; n++)
            for (Node *a = n->parent; a; a = a->parent) depth++;
        LARGE_INTEGER e; QueryPerformanceCounter(&e); t_walk += (double)(e.QuadPart - s.QuadPart) * 1000.0 / qf.QuadPart; }
        { LARGE_INTEGER s; QueryPerformanceCounter(&s);
        for (uint32_t i = 1; i < N; i++) {
            if ((i + p) % 100 != 0) continue;
            Node *c = &nodes[i], *np = &nodes[h_rand() % i], *op = c->parent;
            uint32_t k = 0;
            while (op->ch[k] != c) k++;
            memmove(&op->ch[k], &op->ch[k + 1], sizeof(Node *) * (op->nch - k - 1));
            op->nch--;
            add_child(np, c);
        }
        LARGE_INTEGER e; QueryPerformanceCounter(&e); t_rep += (double)(e.QuadPart - s.QuadPart) * 1000.0 / qf.QuadPart; }
    }
    fprintf(stderr, "build=%.1f walk=%.1f reparent=%.1f\n", t_build, t_walk, t_rep);
    uint64_t sum = depth;
    for (uint32_t i = 0; i < N; i++) sum += (uint64_t)((int64_t)nodes[i].value * nodes[i].nch);
    h_report("w2-plain-keep", sum);
    return 0;
}
