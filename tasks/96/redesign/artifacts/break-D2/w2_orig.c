/* D2 (generational references + stable stores), workload W2: a tree with parent pointers.
   What the D2 compiler would emit for tasks/96/redesign/phase3-design-D2.md section 3, W2:
   - every Node lives in the Node store: a reserved address range of 40-byte slots that never move,
     each starting with a 32-bit generation (odd = live); slot 0 is reserved so None = {0, 0};
   - a reference to a Node is 8 bytes, {slot index, generation}; deref = store base + index * 40;
   - `nodes: list[own Node]` owns every Node; `parent` and `children` hold references;
   - proof P3 (grow-only owner): Node objects are owned only by `nodes`, which main only appends to,
     so every non-None Node reference is live while main runs: no generation check anywhere;
     `n.parent is not None` is a test of the reference's generation against 0;
   - indexing `nodes` with a random index keeps its bounds check; `for n in nodes` does not need one;
   - lists are Ember Arrays: {ptr, len, cap}, doubling from 4 ([HEAP-2]).
   Not modelled (outside the memory model, same for every design): integer overflow checks, and the
   drop of `nodes` at the end of main, which runs after h_report. */
#include "../bench/harness.h"
#include <string.h>

typedef uint32_t u32;
typedef int32_t i32;
typedef uint64_t u64;
typedef int64_t i64;

#define N 1000000u
#define PASSES 10u

static void d2_panic(const char *m) {
    fprintf(stderr, "panic: %s\n", m);
    fflush(stderr);
    ExitProcess(101);
}

typedef struct { char *base; size_t reserved, committed; } Range;

static void range_init(Range *r, size_t reserve) {
    r->base = (char *)VirtualAlloc(NULL, reserve, MEM_RESERVE, PAGE_NOACCESS);
    if (!r->base) d2_panic("address space reservation failed");
    r->reserved = reserve;
    r->committed = 0;
}

static void range_ensure(Range *r, size_t bytes) {
    if (bytes <= r->committed) return;
    size_t chunk = (size_t)1 << 20;
    size_t want = (bytes + chunk - 1) & ~(chunk - 1);
    if (want > r->reserved) d2_panic("store capacity exhausted");
    if (!VirtualAlloc(r->base + r->committed, want - r->committed, MEM_COMMIT, PAGE_READWRITE))
        d2_panic("out of memory");
    r->committed = want;
}

typedef struct { u32 idx, gen; } Ref; /* a generational reference; None = {0, 0} */

typedef struct { Ref *ptr; i64 len, cap; } RefList; /* list[Node] */

static void reflist_push(RefList *l, Ref r) {
    if (l->len == l->cap) {
        i64 nc = l->cap ? l->cap * 2 : 4;
        Ref *np = (Ref *)realloc(l->ptr, (size_t)nc * sizeof(Ref));
        if (!np) d2_panic("out of memory");
        l->ptr = np;
        l->cap = nc;
    }
    l->ptr[l->len++] = r;
}

/* list.remove(x): the first element equal to x (same object), order kept */
static void reflist_remove(RefList *l, Ref r) {
    for (i64 i = 0; i < l->len; i++) {
        if (l->ptr[i].idx == r.idx && l->ptr[i].gen == r.gen) {
            memmove(&l->ptr[i], &l->ptr[i + 1], (size_t)(l->len - i - 1) * sizeof(Ref));
            l->len--;
            return;
        }
    }
    d2_panic("list.remove(x): x not in list");
}

typedef struct {
    u32 gen;
    i32 value;
    Ref parent;       /* Option[Node], a reference */
    RefList children; /* list[Node], references */
} NodeSlot;           /* 40 bytes */

typedef struct { Range r; u32 len; } Store; /* W2 never destroys a Node, so no free list is touched */

static Store nodes_store;

static inline NodeSlot *node(u32 idx) { return (NodeSlot *)nodes_store.r.base + idx; }

static void store_init(Store *s) {
    range_init(&s->r, (size_t)256 << 20);
    range_ensure(&s->r, sizeof(NodeSlot));
    ((NodeSlot *)s->r.base)[0].gen = 2; /* slot 0 is reserved: a dead generation, never allocated */
    s->len = 1;
}

static Ref node_new(i32 value, Ref parent) {
    u32 s = nodes_store.len++;
    range_ensure(&nodes_store.r, (size_t)nodes_store.len * sizeof(NodeSlot));
    NodeSlot *n = node(s);
    n->gen += 1; /* odd: live */
    n->value = value;
    n->parent = parent;
    n->children = (RefList){0};
    return (Ref){s, n->gen};
}

int main(void) {
    h_start();
    h_seed(42);
    store_init(&nodes_store);

    RefList nodes = {0}; /* nodes: list[own Node] */
    reflist_push(&nodes, node_new(0, (Ref){0, 0}));
    for (u32 i = 1; i < N; i++) {
        u32 pi = h_rand() % i;
        if ((i64)pi >= nodes.len) d2_panic("index out of range");
        Ref p = nodes.ptr[pi];
        Ref n = node_new((i32)(i % 1000), p);
        reflist_push(&nodes, n);           /* nodes owns n */
        reflist_push(&node(p.idx)->children, n); /* p is live (P3): no check */
    }

    u64 depth_sum = 0;
    for (u32 pass = 0; pass < PASSES; pass++) {
        /* (a) for n in nodes: walk the parent references to the root */
        for (i64 i = 0; i < nodes.len; i++) {
            NodeSlot *m = node(nodes.ptr[i].idx);
            u64 d = 0;
            while (m->parent.gen != 0) { /* `m.parent is not None`; live by P3 */
                m = node(m->parent.idx);
                d++;
            }
            depth_sum += d;
        }
        /* (b) re-parent every hundredth node */
        for (u32 i = 1; i < N; i++) {
            if ((i + pass) % 100 != 0) continue;
            u32 qi = h_rand() % i;
            if ((i64)i >= nodes.len || (i64)qi >= nodes.len) d2_panic("index out of range");
            Ref n = nodes.ptr[i];
            Ref q = nodes.ptr[qi];
            NodeSlot *ns = node(n.idx);
            if (ns->parent.gen == 0) d2_panic("None has no attribute `children`");
            reflist_remove(&node(ns->parent.idx)->children, n);
            ns->parent = q;
            reflist_push(&node(q.idx)->children, n);
        }
    }

    u64 sum = depth_sum;
    for (i64 i = 0; i < nodes.len; i++) {
        NodeSlot *ns = node(nodes.ptr[i].idx);
        sum += (u64)((i64)ns->value * ns->children.len);
    }

    h_report("D2-w2", sum);
    return 0;
}
