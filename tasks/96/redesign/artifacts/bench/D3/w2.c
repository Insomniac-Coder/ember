/* D3 model of W2 (tree with parent pointers), as the D3 compiler would emit the program in
   tasks/96/redesign/phase3-design-D3.md (section 3, W2).

   What D3 puts here, and nothing else:
   - `nodes` (a List local to main) is the HOME of every Node: each node is first stored there.
     `parent` and the `children` elements are WATCHING references.
   - Whole-program ownership facts: Node's only home is `nodes`; `nodes` is append-only for its whole
     life (no remove, pop, clear or reassignment); every Node watcher lives in a Node or in a local
     of main inside `nodes`' scope. So no watcher can outlive its target: every watcher is a plain
     pointer, with no generation word and no check. Node is final, never cast and never used through
     an interface: no type index. So Node has NO object header.
   - Objects come from the runtime's object slab (per size class; bump allocation, as nothing is
     destroyed before main ends). Lists are the plain 24-byte List (none is ever pinned).
   - Checks kept: bounds checks the compiler cannot prove (`nodes[h_rand() % i]`, `nodes[i]`),
     the unwrap of `node.parent` before `.children` (None -> panic), `remove` finding its element.
   - The final drop of `nodes` happens when main's scope ends, after h_report: not in the timing.
   Left out on purpose, identical in every memory model: integer overflow checks ([TYP-8]). */
#include "../harness.h"
#include <string.h>

#define N 1000000
#define PASSES 10

static void panic(const char *m) { fprintf(stderr, "panic: %s\n", m); exit(101); }
#define CHECK(c, m) do { if (__builtin_expect(!(c), 0)) panic(m); } while (0)
static void *xrealloc(void *p, size_t n) { void *q = realloc(p, n); CHECK(q, "out of memory"); return q; }

typedef struct Node Node;
typedef struct { Node **data; int64_t len, cap; } ListNode; /* List[Node]: plain header */
struct Node {                 /* no header (see above) */
    int32_t value;
    Node *parent;             /* Node? watcher: plain pointer, NULL = None */
    ListNode children;        /* List[Node] of watchers */
};

/* the runtime's object slab for this size class */
static char *slab_cur, *slab_end;
static Node *node_alloc(void) {
    if (slab_cur == slab_end) {
        size_t sz = (size_t)65536 * sizeof(Node);
        slab_cur = xrealloc(NULL, sz);
        slab_end = slab_cur + sz;
    }
    Node *n = (Node *)slab_cur;
    slab_cur += sizeof(Node);
    return n;
}

static inline void list_push(ListNode *l, Node *n) {
    if (l->len == l->cap) {
        int64_t c = l->cap ? l->cap * 2 : 4;
        l->data = xrealloc(l->data, (size_t)c * sizeof(Node *));
        l->cap = c;
    }
    l->data[l->len++] = n;
}
static inline Node *list_get(ListNode *l, int64_t i) {
    CHECK(i >= 0 && i < l->len, "index out of bounds");
    return l->data[i];
}
/* list.remove(x): the first element that is x (objects compare by identity), then close the gap */
static void list_remove(ListNode *l, Node *x) {
    int64_t i = 0;
    while (i < l->len && l->data[i] != x) i++;
    CHECK(i < l->len, "remove(x): x not in list");
    memmove(&l->data[i], &l->data[i + 1], (size_t)(l->len - i - 1) * sizeof(Node *));
    l->len--;
}

int main(void) {
    h_start();
    h_seed(42);
    ListNode nodes = {0};
    Node *root = node_alloc();
    root->value = 0; root->parent = NULL; root->children = (ListNode){0};
    list_push(&nodes, root);
    for (int64_t i = 1; i < N; i++) {
        Node *p = list_get(&nodes, (int64_t)h_rand() % i);
        Node *n = node_alloc();
        n->value = (int32_t)(i % 1000); n->parent = p; n->children = (ListNode){0};
        list_push(&nodes, n);            /* the home */
        list_push(&p->children, n);      /* a watcher */
    }

    uint64_t depth_sum = 0;
    for (int64_t pass = 0; pass < PASSES; pass++) {
        /* for n in nodes: d = n; while p := d.parent: depth_sum += 1; d = p */
        Node **nd = nodes.data;
        int64_t nn = nodes.len;
        for (int64_t i = 0; i < nn; i++) {
            Node *d = nd[i];
            Node *p;
            while ((p = d->parent) != NULL) { depth_sum++; d = p; }
        }
        /* for i in range(1, N): if (i + pass) % 100 == 0: ... */
        for (int64_t i = 1; i < N; i++) {
            if ((i + pass) % 100 != 0) continue;
            int64_t np = (int64_t)h_rand() % i;
            Node *node = list_get(&nodes, i);
            Node *old = node->parent;
            CHECK(old != NULL, "None has no field `children`");
            list_remove(&old->children, node);
            Node *newp = list_get(&nodes, np);
            node->parent = newp;
            list_push(&newp->children, node);
        }
    }

    uint64_t sum = depth_sum;
    for (int64_t i = 0; i < nodes.len; i++) {
        Node *n = nodes.data[i];
        sum += (uint64_t)((int64_t)n->value * n->children.len);
    }
    h_report("D3-w2", sum);
    return 0;
}
