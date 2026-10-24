/* W2, plain C written the way a class-based program is laid out: one malloc per node, a list of node
   pointers, order-keeping removal from the old parent. No header, no counts, no checks. This is the
   "same work, same way" baseline for the Ember models. */
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
    Node **nodes = malloc(sizeof(Node *) * N);
    nodes[0] = calloc(1, sizeof(Node));
    for (uint32_t i = 1; i < N; i++) {
        Node *n = calloc(1, sizeof(Node));
        n->value = (int32_t)(i % 1000);
        nodes[i] = n;
        add_child(nodes[h_rand() % i], n);
    }
    LARGE_INTEGER qa, qf; QueryPerformanceFrequency(&qf); QueryPerformanceCounter(&qa);
    double t_build = (double)(qa.QuadPart - h_t0.QuadPart) * 1000.0 / qf.QuadPart, t_walk = 0, t_rep = 0;
    uint64_t depth = 0;
    for (uint32_t p = 0; p < 10; p++) {
        { LARGE_INTEGER s; QueryPerformanceCounter(&s);
        for (uint32_t j = 0; j < N; j++)
            for (Node *a = nodes[j]->parent; a; a = a->parent) depth++;
        LARGE_INTEGER e; QueryPerformanceCounter(&e); t_walk += (double)(e.QuadPart - s.QuadPart) * 1000.0 / qf.QuadPart; }
        { LARGE_INTEGER s; QueryPerformanceCounter(&s);
        for (uint32_t i = 1; i < N; i++) {
            if ((i + p) % 100 != 0) continue;
            Node *c = nodes[i], *np = nodes[h_rand() % i], *op = c->parent;
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
    for (uint32_t i = 0; i < N; i++) sum += (uint64_t)((int64_t)nodes[i]->value * nodes[i]->nch);
    h_report("w2-plain-malloc", sum);
    return 0;
}
