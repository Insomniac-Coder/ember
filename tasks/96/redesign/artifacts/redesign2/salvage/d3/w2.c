/* "today's rules, but never an error" (phase3-design-D3.md), W2: a tree with parent pointers.
   The default build is target/redesign/bench/D3/w2.c unchanged (the PROVEN case: `nodes` is the only
   home of every Node and is append-only, so every watcher is a raw pointer and Node has no header).
   Built with -DUNPROVEN: D3's own fallback, which phase 3 did not model and the breaker said one
   `nodes.pop()` anywhere triggers (break 2.8): Node gets a 4-byte generation, `parent` becomes a
   12-byte watcher {pointer, generation} packed after it, every `children` element a 16-byte watcher,
   and every hop of the walk compares the generation (a dead parent would read as None).
   Left out, as in every model: [TYP-8] overflow checks; the final drop after h_report. */
#include "../sh.h"
#include <string.h>

#define N 1000000
#define PASSES 10
static void panic(const char *m) { fprintf(stderr, "panic: %s\n", m); exit(101); }
#define CHECK(c, m) do { if (__builtin_expect(!(c), 0)) panic(m); } while (0)
static void *xrealloc(void *p, size_t n) { void *q = realloc(p, n); CHECK(q, "out of memory"); return q; }

typedef struct Node Node;
#ifdef UNPROVEN
typedef struct { Node *p; uint32_t gen; } Watch;              /* 16 bytes in an array, 12 packed in Node */
typedef struct { Watch *data; int64_t len, cap; } ListW;
struct Node { uint32_t gen; int32_t value; Node *parent; uint32_t parent_gen; ListW children; }; /* 48 bytes */
static inline Node *watched(Node *p, uint32_t g) { return (p && p->gen == g) ? p : NULL; }
#define ELEM Watch
#define SAME(w, n) ((w).p == (n) && (w).gen == (n)->gen)
#else
typedef struct { Node **data; int64_t len, cap; } ListW;      /* List[Node]: plain header */
struct Node { int32_t value; Node *parent; ListW children; };  /* 40 bytes, no header */
#define ELEM Node *
#define SAME(w, n) ((w) == (n))
#endif
typedef struct { Node **data; int64_t len, cap; } ListHome;    /* `nodes`, the home */

static char *slab_cur, *slab_end;
static Node *node_alloc(void) {
    if (slab_cur == slab_end) { size_t sz = (size_t)65536 * sizeof(Node); slab_cur = xrealloc(NULL, sz); slab_end = slab_cur + sz; }
    Node *n = (Node *)slab_cur; slab_cur += sizeof(Node);
#ifdef UNPROVEN
    n->gen = 1;
#endif
    return n;
}
static inline void home_push(ListHome *l, Node *n) {
    if (l->len == l->cap) { int64_t c = l->cap ? l->cap * 2 : 4; l->data = xrealloc(l->data, (size_t)c * sizeof(Node *)); l->cap = c; }
    l->data[l->len++] = n;
}
static inline Node *home_get(ListHome *l, int64_t i) { CHECK(i >= 0 && i < l->len, "index out of bounds"); return l->data[i]; }
static inline void list_push(ListW *l, Node *n) {
    if (l->len == l->cap) { int64_t c = l->cap ? l->cap * 2 : 4; l->data = xrealloc(l->data, (size_t)c * sizeof(ELEM)); l->cap = c; }
#ifdef UNPROVEN
    l->data[l->len++] = (Watch){n, n->gen};
#else
    l->data[l->len++] = n;
#endif
}
static void list_remove(ListW *l, Node *x) {
    int64_t i = 0;
    while (i < l->len && !SAME(l->data[i], x)) i++;
    CHECK(i < l->len, "remove(x): x not in list");
    memmove(&l->data[i], &l->data[i + 1], (size_t)(l->len - i - 1) * sizeof(ELEM));
    l->len--;
}
static inline void set_parent(Node *n, Node *p) {
    n->parent = p;
#ifdef UNPROVEN
    n->parent_gen = p ? p->gen : 0;
#endif
}
static inline Node *get_parent(Node *n) {
#ifdef UNPROVEN
    return watched(n->parent, n->parent_gen);                   /* WATCH: one compare per hop */
#else
    return n->parent;
#endif
}

int main(void) {
    h_start();
    h_seed(42);
    ListHome nodes = {0};
    Node *root = node_alloc();
    root->value = 0; set_parent(root, NULL); root->children = (ListW){0};
    home_push(&nodes, root);
    for (int64_t i = 1; i < N; i++) {
        Node *p = home_get(&nodes, (int64_t)h_rand() % i);
        Node *n = node_alloc();
        n->value = (int32_t)(i % 1000); set_parent(n, p); n->children = (ListW){0};
        home_push(&nodes, n);
        list_push(&p->children, n);
    }
    H_PHASE("build");
    uint64_t depth_sum = 0;
    for (int64_t pass = 0; pass < PASSES; pass++) {
        Node **nd = nodes.data;
        int64_t nn = nodes.len;
        for (int64_t i = 0; i < nn; i++) {
            Node *d = nd[i], *p;
            while ((p = get_parent(d)) != NULL) { depth_sum++; d = p; }
        }
        for (int64_t i = 1; i < N; i++) {
            if ((i + pass) % 100 != 0) continue;
            int64_t np = (int64_t)h_rand() % i;
            Node *node = home_get(&nodes, i);
            Node *old = get_parent(node);
            CHECK(old != NULL, "None has no field `children`");
            list_remove(&old->children, node);
            Node *newp = home_get(&nodes, np);
            set_parent(node, newp);
            list_push(&newp->children, node);
        }
    }
    H_PHASE("passes");
    uint64_t sum = depth_sum;
    for (int64_t i = 0; i < nodes.len; i++) {
        Node *n = nodes.data[i];
        sum += (uint64_t)((int64_t)n->value * n->children.len);
    }
#ifdef UNPROVEN
    h_report("d3-w2-unproven", sum);
#else
    h_report("d3-w2", sum);
#endif
    return 0;
}
