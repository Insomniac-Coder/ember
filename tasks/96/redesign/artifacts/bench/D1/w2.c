/* D1 (values and projections), workload W2: a tree with parent pointers.
   Report: tasks/96/redesign/phase3-design-D1.md (sections 3, 4, 5).

   The user program declares `class Node` with `weak parent: Option[Node]` and `children: Array[Node]`.
   What D1 needs and is therefore here:
   - a class object is one heap block: 16-byte header (strong u32, weak u32, type pointer), then
     its fields; `weak` starts at 1 on behalf of all strong handles ([OBJ-3]);
   - an Array value is one pointer to a counted buffer (header: count, shift version, len, cap);
     every in-place change first tests count == 1 (copy-on-write); push tests count and capacity;
   - copying a handle into a list retains it; a handle read from a list and used where nothing can
     release it is not counted ([RC-2b]/[RC-2e]); the handle `old`, held across a release that
     could run deinit code, is retained;
   - reading a weak field tests the target's strong count; a dead target is cleared from the field
     (lazy zeroing), releasing its weak count; storing a weak field adjusts weak counts;
   - every index the compiler cannot prove in range is checked; swap_remove bumps the shift
     version.
   Left out on purpose, the same in every memory model: the integer overflow checks of [TYP-8].
   No teardown at the end: memory-only values whose last use ends main are not freed. */
#include "../harness.h"
#include <string.h>

static _Noreturn void d1_panic(const char *msg) {
    fprintf(stderr, "panic: %s\n", msg);
    fflush(stderr);
    ExitProcess(101);
}
static void *xmalloc(size_t n) {
    void *p = malloc(n);
    if (!p) d1_panic("out of memory");
    return p;
}

/* ---- D1 runtime: counted buffers (Array) ---- */
typedef struct Buf { uint32_t count; uint32_t shifts; uint64_t len; uint64_t cap; } Buf; /* 24 bytes */
static Buf EMPTY = {0x80000000u, 0, 0, 0}; /* the shared empty buffer: never unique, never freed */
#define EL(b, T) ((T *)((Buf *)(b) + 1))
typedef void (*RetainFn)(void *elems, uint64_t n);

static Buf *buf_reserve_slow(Buf **slot, size_t esz, uint64_t need, RetainFn retain) {
    Buf *b = *slot;
    uint64_t cap = b->cap;
    if (need > cap) {
        uint64_t nc = cap < 4 ? 4 : cap * 2;
        while (nc < need) nc *= 2;
        cap = nc;
    }
    if (b->count == 1) {
        b = realloc(b, sizeof(Buf) + cap * esz);
        if (!b) d1_panic("out of memory");
        b->cap = cap;
    } else {
        Buf *nb = xmalloc(sizeof(Buf) + cap * esz);
        nb->count = 1; nb->shifts = 0; nb->len = b->len; nb->cap = cap;
        memcpy(nb + 1, b + 1, b->len * esz);
        if (retain) retain(nb + 1, b->len);
        if (b != &EMPTY) b->count--;
        b = nb;
    }
    *slot = b;
    return b;
}
static inline Buf *buf_mut(Buf **slot, size_t esz, RetainFn retain) {
    Buf *b = *slot;
    if (__builtin_expect(b->count != 1, 0)) b = buf_reserve_slow(slot, esz, b->len, retain);
    return b;
}
#define PUSH(slot, T, val, retain)                                                                   \
    do {                                                                                             \
        Buf *b_ = *(slot);                                                                           \
        if (__builtin_expect(b_->count != 1 || b_->len == b_->cap, 0))                               \
            b_ = buf_reserve_slow((slot), sizeof(T), b_->len + 1, (retain));                         \
        EL(b_, T)[b_->len++] = (val);                                                                \
    } while (0)

/* ---- class Node ---- */
typedef struct TypeInfo { const char *name; } TypeInfo;
typedef struct Node {
    uint32_t strong, weak;     /* header */
    const TypeInfo *type;      /* header */
    int32_t value;
    struct Node *parent;       /* weak */
    Buf *children;             /* Array[Node]: strong handles */
} Node;                        /* 40 bytes */
static const TypeInfo TI_NODE = {"Node"};

static void retain_nodes(void *el, uint64_t n) {
    Node **a = el;
    for (uint64_t i = 0; i < n; i++) a[i]->strong++;
}
static inline void weak_release(Node *p) {
    if (--p->weak == 0) free(p);
}
static void node_release(Node *n);
static void node_deinit(Node *n) { /* strong reached 0: drop the fields, then the strong owners' weak unit */
    Buf *c = n->children;
    if (c != &EMPTY && --c->count == 0) {
        for (uint64_t i = 0; i < c->len; i++) node_release(EL(c, Node *)[i]);
        free(c);
    }
    if (n->parent) weak_release(n->parent);
    weak_release(n);
}
static inline void node_release(Node *n) {
    if (--n->strong == 0) node_deinit(n);
}
/* read a weak field: the target if it is alive, else None (and the field is cleared) */
static inline Node *weak_get(Node **field) {
    Node *p = *field;
    if (p && __builtin_expect(p->strong == 0, 0)) {
        *field = NULL;
        weak_release(p);
        p = NULL;
    }
    return p;
}
/* store a weak field */
static inline void weak_set(Node **field, Node *v) {
    Node *old = *field;
    if (v) v->weak++;
    *field = v;
    if (old) weak_release(old);
}
static Node *node_new(int32_t value, Node *parent) { /* Node(value=..., parent=...) */
    Node *n = xmalloc(sizeof *n);
    n->strong = 1; n->weak = 1; n->type = &TI_NODE;
    n->value = value;
    n->parent = NULL;
    weak_set(&n->parent, parent);
    n->children = &EMPTY;
    return n;
}

int main(void) {
    h_start();
    h_seed(42);
    enum { N = 1000000 };
    Buf *nodes = &EMPTY;
    PUSH(&nodes, Node *, node_new(0, NULL), retain_nodes); /* a temporary: moved in */

    for (uint32_t i = 1; i < N; i++) {
        uint32_t r = h_rand() % i;
        if (r >= nodes->len) d1_panic("index out of bounds");
        Node *parent = EL(nodes, Node *)[r]; /* not counted: nothing can release it below */
        Node *child = node_new((int32_t)(i % 1000), parent);
        child->strong++; /* copied into parent.children: retain */
        PUSH(&parent->children, Node *, child, retain_nodes);
        PUSH(&nodes, Node *, child, retain_nodes); /* last use of child: a move */
    }

    uint64_t depth_sum = 0;
    for (uint32_t p = 0; p < 10; p++) {
        /* (a) `for node in nodes: n = node; while Some(up) = n.parent: depth_sum += 1; n = up` */
        const Buf *nb = nodes; /* nodes is not changed in this loop: hoisted, no re-fetch */
        for (uint64_t i = 0; i < nb->len; i++) {
            Node *n = EL(nb, Node *)[i];
            uint64_t d = 0;
            for (;;) {
                Node *up = weak_get(&n->parent);
                if (!up) break;
                d++;
                n = up;
            }
            depth_sum += d;
        }
        /* (b) reparenting */
        for (uint32_t i = 1; i < N; i++) {
            if ((i + p) % 100 != 0) continue;
            uint32_t r = h_rand() % i;
            if (r >= nodes->len) d1_panic("index out of bounds");
            Node *new_parent = EL(nodes, Node *)[r];
            if (i >= nodes->len) d1_panic("index out of bounds");
            Node *child = EL(nodes, Node *)[i];
            /* `if Some(old) = child.parent: old.children.swap_remove(old.children.index_of(child))` */
            Node *old = weak_get(&child->parent);
            if (old) {
                old->strong++; /* held across a release that may run deinit code */
                const Buf *cb = old->children;
                uint64_t j = 0;
                while (j < cb->len && EL(cb, Node *)[j] != child) j++;
                if (j == cb->len) d1_panic("index_of: not in the list");
                Buf *mb = buf_mut(&old->children, sizeof(Node *), retain_nodes);
                Node *removed = EL(mb, Node *)[j]; /* swap_remove(j): j < len, found above */
                EL(mb, Node *)[j] = EL(mb, Node *)[mb->len - 1];
                mb->len--; mb->shifts++;
                node_release(removed); /* the returned handle is dropped */
                node_release(old);
            }
            weak_set(&child->parent, new_parent);   /* child.parent = new_parent */
            child->strong++;                        /* new_parent.children.push(child): a copy */
            PUSH(&new_parent->children, Node *, child, retain_nodes);
        }
    }

    uint64_t total = depth_sum;
    for (uint64_t i = 0; i < nodes->len; i++) {
        const Node *n = EL(nodes, Node *)[i];
        total += (uint64_t)((int64_t)n->value * (int64_t)n->children->len);
    }
    h_report("D1-w2", total);
    return 0;
}
