/* W1 "trees", salvage (b) of verdict 47: memory laid out so a block's address finds its arena.

   The idea: the arena takes its chunks on 64 KB boundaries (VirtualAlloc's granularity on Windows;
   an aligned mmap or _aligned_malloc elsewhere) and writes its own address in each chunk's first 64
   bytes. A block's instance is then `((HChunk *)(p & ~0xFFFF))->arena`: one AND and one load, done
   only where the instance is needed (growth, a free). The value keeps no word: an
   `owns Expr using Arena` place is 8 bytes, an inner node 24.

   Unlike salvage (a), the rule's behaviour is kept whole: each node's end still runs and still
   gives its block back to the instance it came from, found from the block's address instead of a
   stored word ([ARN-2], R3's purpose: "so that its growth and its end go back to the instance it
   came from"). `Arena`'s `deallocate` is empty, so after inlining the AND and the load are dead and
   what stays is the walk; that isolates the walk's cost (this program) from the word's (against
   model_trees_inl) and from both (against salvage_trees_nofree).

   What it needs from the language: nothing in the program's text. The `Allocator` interface gains
   one more unsafe promise a kind may make, e.g. `fn instance_of(p: *mut u8) -> sharedref mut Self`
   (declared, so the compiler stores no word for that kind's values), which the kind's author proves
   by its layout; a kind that cannot (Global, an allocator over a foreign heap) does not make it and
   its values keep the word. The kind's chunks must be aligned to their size, and a block bigger
   than a chunk gets a region of its own whose header precedes it (m3_rt.c, h_arena_slow).

   Safety kept, exactly: the lifetime check (E3063, [LT-4]) and [ALC-5]'s reset rule read the making
   line, not the word, so they are unchanged; the instance found is the one the block came from by
   the kind's promise (the same kind of promise `Allocator`'s implementer already makes, [ALC-2]:
   "valid, aligned, unaliased memory"). Cost: 64 bytes per 64 KB chunk (0.1%). */
#include "harness.h"
#include "m3_rt.h"

#define NB (1u << 20)
#define BATCHES 16
#define DEPTH 12

typedef struct Expr Expr;
struct Expr { int32_t tag, val; Expr *l, *r; }; /* `owns Expr using Arena`: 8 bytes, no word */
typedef char size_check_Expr[sizeof(Expr) == 24 ? 1 : -1]; /* the layout M3.md states */

static inline void deallocate_inl(HArena *inst, void *p, size_t size, size_t align) {
    (void)inst; (void)p; (void)size; (void)align; /* [ARN-2]: frees nothing one by one */
}

static Expr *new_expr(HArena *a) {
    Expr *p = (Expr *)h_arena_bump(a, sizeof(Expr), 8); /* arena.allocate, inline */
    if (p == NULL) m3_panic("out of memory");            /* [ALC-4] */
    return p;
}

static Expr *build(HArena *a, int d, uint64_t *count) {
    uint32_t r = h_rand();
    Expr *e;
    if (d == 0 || (r & 3) == 0) {
        e = new_expr(a);
        e->tag = 0; e->val = (int32_t)((r >> 2) % 100);
    } else {
        Expr *l = build(a, d - 1, count);
        Expr *rr = build(a, d - 1, count);
        e = new_expr(a);
        e->tag = (int32_t)(1 + (r >> 2) % 3); e->val = 0; e->l = l; e->r = rr;
    }
    ++*count;
    return e;
}

static uint32_t eval(const Expr *e) {
    switch (e->tag) {
    case 0: return (uint32_t)e->val;
    case 1: return eval(e->l) + eval(e->r);
    case 2: return eval(e->l) * eval(e->r);
    default: return eval(e->l) - eval(e->r);
    }
}

/* The end of an `owns Expr using Arena`: Expr's drop, then the block back to the instance found
   from its address ([ARN-2], R3). */
static void drop_expr(Expr *e) {
    if (e->tag != 0) { drop_expr(e->l); drop_expr(e->r); }
    deallocate_inl(h_arena_of(e), e, sizeof(Expr), 8);
}

int main(void) {
    h_start();
    h_seed(42);
    uint64_t cs = 0;
    HArena *arena = h_arena_new();
    for (int b = 0; b < BATCHES; b++) {
        size_t nroots = 0, cap = 0;
        Expr **roots = NULL; /* Array[owns Expr using Arena]: 8 bytes per element */
        uint64_t count = 0;
        while (count < NB) {
            if (nroots == cap) { cap = cap ? cap * 2 : 4; roots = (Expr **)realloc(roots, cap * sizeof(Expr *)); }
            roots[nroots++] = build(arena, DEPTH, &count);
        }
        for (size_t i = 0; i < nroots; i++) cs = cs * 1000003u + eval(roots[i]);
        for (size_t i = 0; i < nroots; i++) drop_expr(roots[i]); /* mem.drop(roots) */
        free(roots);
        h_arena_reset(arena);
        cs += count;
    }
    h_arena_free(arena);
    h_report("salvage_trees_chunkhdr", cs);
    return 0;
}
