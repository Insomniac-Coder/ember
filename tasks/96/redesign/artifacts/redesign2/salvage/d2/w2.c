/* SALVAGED "checked pointers" (phase3-design-D2.md), W2: a tree with parent pointers.
   Built from target/redesign/bench/D2/w2.c (Node store in a reserved address range whose slots
   never move, a 32-bit generation per slot, proof P3 removing every check, bounds checks kept).
   Fixes:
   - FIX 1 (the reference): a reference is ONE 64-bit word, {48-bit slot address, 16-bit generation}
     (x64 user addresses fit in 47 bits). Following it is one AND, where {32-bit slot index, 32-bit
     generation} needed base + index * 40 on every hop of the parent walk, a latency-bound chain.
     A slot whose 16-bit generation runs out is retired (as [HND-3] does today), so a stale
     reference can never match a new occupant. Still 8 bytes, checked or not.
   - FIX 2 (owned fields inline): a field initialised with a new object and never re-pointed (here
     `children: list[Node] = []`) lives inside its owner's slot; the breaker's faithful version kept
     it as a separate list object (one more load per use and +21% memory, phase3-measurements.md).
   Built with -DCHECKED: every hop checks the generation, as when P3 does not hold (one
   `nodes.pop()` anywhere in the program); that build prices the fallback.
   Left out, as in every model: [TYP-8] overflow checks; the drop of `nodes` after h_report. */
#include "../sh.h"
#include <string.h>

#define N 1000000u
#define PASSES 10u
static void d2_panic(const char *m) { fprintf(stderr, "panic: %s\n", m); fflush(stderr); ExitProcess(101); }

typedef struct { char *base; size_t reserved, committed; } Range;
static void range_init(Range *r, size_t reserve) {
    r->base = (char *)VirtualAlloc(NULL, reserve, MEM_RESERVE, PAGE_NOACCESS);
    if (!r->base) d2_panic("address space reservation failed");
    r->reserved = reserve; r->committed = 0;
}
static void range_ensure(Range *r, size_t bytes) {
    if (bytes <= r->committed) return;
    size_t chunk = (size_t)1 << 20, want = (bytes + chunk - 1) & ~(chunk - 1);
    if (want > r->reserved) d2_panic("store capacity exhausted");
    if (!VirtualAlloc(r->base + r->committed, want - r->committed, MEM_COMMIT, PAGE_READWRITE)) d2_panic("out of memory");
    r->committed = want;
}

typedef uint64_t GRef;                         /* {48-bit address, 16-bit generation}; None = 0 */
#define GADDR(r) ((r) & 0x0000FFFFFFFFFFFFull)
#define GGEN(r) ((uint32_t)((r) >> 48))
typedef struct { GRef *ptr; int64_t len, cap; } RefList;
typedef struct NodeSlot { uint32_t gen; int32_t value; GRef parent; RefList children; } NodeSlot; /* 40 bytes */
_Static_assert(sizeof(NodeSlot) == 40, "slot layout");
#define NODE(r) ((NodeSlot *)GADDR(r))
static inline GRef gref(NodeSlot *s) { return (GRef)(uintptr_t)s | ((GRef)(s->gen & 0xFFFF) << 48); }
static inline int live(GRef r) { return r && (NODE(r)->gen & 0xFFFF) == GGEN(r); }

static void reflist_push(RefList *l, GRef r) {
    if (l->len == l->cap) {
        int64_t nc = l->cap ? l->cap * 2 : 4;
        GRef *np = realloc(l->ptr, (size_t)nc * sizeof(GRef));
        if (!np) d2_panic("out of memory");
        l->ptr = np; l->cap = nc;
    }
    l->ptr[l->len++] = r;
}
static void reflist_remove(RefList *l, GRef r) { /* list.remove(x): first equal element, order kept */
    for (int64_t i = 0; i < l->len; i++)
        if (l->ptr[i] == r) {
            memmove(&l->ptr[i], &l->ptr[i + 1], (size_t)(l->len - i - 1) * sizeof(GRef));
            l->len--;
            return;
        }
    d2_panic("list.remove(x): x not in list");
}

static Range store; static uint32_t store_len;  /* the Node store; nothing is destroyed in W2 */
static GRef node_new(int32_t value, GRef parent) {
    uint32_t s = store_len++;
    range_ensure(&store, (size_t)store_len * sizeof(NodeSlot));
    NodeSlot *n = (NodeSlot *)store.base + s;
    n->gen += 1; /* odd: live */
    n->value = value; n->parent = parent; n->children = (RefList){0};
    return gref(n);
}

int main(void) {
    h_start();
    h_seed(42);
    range_init(&store, (size_t)256 << 20);
    RefList nodes = {0}; /* nodes: list[own Node] */
    reflist_push(&nodes, node_new(0, 0));
    for (uint32_t i = 1; i < N; i++) {
        uint32_t pi = h_rand() % i;
        if ((int64_t)pi >= nodes.len) d2_panic("index out of range");
        GRef p = nodes.ptr[pi];
        GRef n = node_new((int32_t)(i % 1000), p);
        reflist_push(&nodes, n);
        reflist_push(&NODE(p)->children, n);       /* P3: p is live */
    }
    H_PHASE("build");
    uint64_t depth_sum = 0;
    for (uint32_t pass = 0; pass < PASSES; pass++) {
        for (int64_t i = 0; i < nodes.len; i++) {
            NodeSlot *m = NODE(nodes.ptr[i]);
            uint64_t d = 0;
#ifdef CHECKED
            for (GRef r = m->parent; live(r); r = m->parent) { m = NODE(r); d++; } /* stale reads as None */
#else
            while (m->parent != 0) { m = NODE(m->parent); d++; }                   /* P3: no check */
#endif
            depth_sum += d;
        }
        for (uint32_t i = 1; i < N; i++) {
            if ((i + pass) % 100 != 0) continue;
            uint32_t qi = h_rand() % i;
            if ((int64_t)i >= nodes.len || (int64_t)qi >= nodes.len) d2_panic("index out of range");
            GRef n = nodes.ptr[i], q = nodes.ptr[qi];
            NodeSlot *ns = NODE(n);
            if (ns->parent == 0) d2_panic("None has no attribute `children`");
            reflist_remove(&NODE(ns->parent)->children, n);
            ns->parent = q;
            reflist_push(&NODE(q)->children, n);
        }
    }
    H_PHASE("passes");
    uint64_t sum = depth_sum;
    for (int64_t i = 0; i < nodes.len; i++) {
        NodeSlot *ns = NODE(nodes.ptr[i]);
        sum += (uint64_t)((int64_t)ns->value * ns->children.len);
    }
#ifdef CHECKED
    h_report("d2-w2-checked", sum);
#else
    h_report("d2-w2", sum);
#endif
    return 0;
}
