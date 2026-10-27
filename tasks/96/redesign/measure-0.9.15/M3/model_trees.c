/* W1 "trees", the model: what Ember emits for this program under spec 0.9.15_Hardened_1 as written.

   The Ember program:

       enum Expr:
           Leaf(val: int32)
           Bin(op: int32, left: owns Expr using Arena, right: owns Expr using Arena)

       fn build(ref mut arena: Arena, depth: int, ref mut count: int) -> owns Expr using Arena:
           r = rand()
           if depth == 0 or r & 3 == 0:
               count += 1
               return owns Expr.Leaf(val=(r >> 2) % 100) using arena
           left = build(ref mut arena, depth - 1, ref mut count)
           right = build(ref mut arena, depth - 1, ref mut count)
           count += 1
           return owns Expr.Bin(op=1 + (r >> 2) % 3, left=left, right=right) using arena

       fn main():
           arena = Arena.with_capacity(1 << 20)
           for b in 0..16:
               count = 0
               roots: Array[owns Expr using Arena] = []     # ends at the round's end, before the reset
               while count < NB:
                   roots.push(build(ref mut arena, 12, ref mut count))
               for e in roots: cs = cs * 1000003 + eval(e)
               mem.drop(roots)       # every tree ends: its drop, each block back to `arena`
               arena.reset()         # allowed: nothing made in `arena` lives ([ALC-5], E3021 otherwise)
               cs += count

   What the rule makes it emit, against plain_trees_arena.c:
   - each `owns Expr using Arena` place keeps its instance's address beside the block's: R3 ("a value
     whose manager may have several instances keeps its instance's address ... one word"), [ALC-1],
     verdict 47. `Arena` may have several instances, so the word is there. An inner node is then
     8 + 2 * 16 = 40 bytes, against C's 24; a root in the list 16 bytes, against 8.
   - each node's block comes from `arena.allocate(Layout(40, 8))` ([ALC-1], [ALC-2]); a null result
     panics "out of memory" ([ALC-4], [HEAP-1]).
   - each node's end gives its block back through the kept word: `deallocate(word, p, Layout(40, 8))`
     ([ARN-2]: "whose end runs its drop and gives its block back to the arena, which frees nothing
     one by one"), so ending a tree walks it ([HEAP-1], R1: a value always ends with its clean-up).
   - `Layout(40, 8)`'s checks ([ALC-2]) are on constants and proved away when compiling.

   Two forms, one source:
   - default ("model_trees"): `Arena` is compiler-known and its operations are runtime calls
     (std/src/arena.em: "The arena operations themselves are the compiler's"), so `allocate` is a
     call of the runtime's ember_arena_alloc_uninit and `deallocate` a call of an empty runtime
     function, both in another translation unit (m3_rt.c), as today;
   - -DM3_INLINE ("model_trees_inl"): the same arena with its bump and its empty `deallocate`
     compiled into the program, the best this rule's emission can do;
   - -DM3_INLINE -DM3_NOWALK ("model_trees_inl_nowalk"): the word kept, as R3 says, but the walk
     that ends each tree proved away: `Arena` is compiler-known and [ARN-2] says its `deallocate`
     frees nothing, and `Expr` has no drop of its own, so ending a tree runs nothing. The spec does
     not say the compiler must find this, so both forms are written. */
#include "harness.h"
#include "m3_rt.h"

#define NB (1u << 20)
#define BATCHES 16
#define DEPTH 12

/* Ember's `Arena` value: a struct holding the runtime's arena ([ARN-1]). The kept word is the
   address of this value, the instance (R3). */
typedef struct Arena { RtArena *rt; } Arena;

#ifdef M3_INLINE
#define ALLOCATE(inst, size, align) rt_arena_bump((inst)->rt, (size), (align))
#ifndef M3_NOWALK
static inline void deallocate_inl(Arena *inst, void *p, size_t size, size_t align) {
    (void)inst; (void)p; (void)size; (void)align; /* [ARN-2]: frees nothing one by one */
}
#endif
#define DEALLOCATE(inst, p, size, align) deallocate_inl((inst), (p), (size), (align))
#ifdef M3_NOWALK
#define NAME "model_trees_inl_nowalk"
#else
#define NAME "model_trees_inl"
#endif
#else
#define ALLOCATE(inst, size, align) rt_arena_alloc_uninit((inst)->rt, 1, (size), (align))
#define DEALLOCATE(inst, p, size, align) rt_arena_deallocate((inst)->rt, (p), (size), (align))
#define NAME "model_trees"
#endif

typedef struct Expr Expr;
/* `owns Expr using Arena`: the block, and its instance's address (R3, [ALC-1], verdict 47). */
typedef struct ExprBox { Expr *p; Arena *a; } ExprBox;
typedef char size_check_ExprBox[sizeof(ExprBox) == 16 ? 1 : -1]; /* the layout M3.md states */
struct Expr { int32_t tag, val; ExprBox l, r; }; /* tag 0 Leaf(val); 1..3 Bin(op = tag): 40 bytes */
typedef char size_check_Expr[sizeof(Expr) == 40 ? 1 : -1]; /* the layout M3.md states */

static Expr *new_expr(Arena *a) {
    Expr *p = (Expr *)ALLOCATE(a, sizeof(Expr), 8); /* `using arena`: arena.allocate ([ALC-1]) */
    if (p == NULL) m3_panic("out of memory");        /* [ALC-4]: a null block panics */
    return p;
}

static ExprBox build(Arena *a, int d, uint64_t *count) {
    uint32_t r = h_rand();
    ExprBox out;
    if (d == 0 || (r & 3) == 0) {
        Expr *e = new_expr(a);
        e->tag = 0; e->val = (int32_t)((r >> 2) % 100);
        out.p = e; out.a = a;                                   /* the word kept (R3) */
    } else {
        ExprBox l = build(a, d - 1, count);
        ExprBox rr = build(a, d - 1, count);
        Expr *e = new_expr(a);
        e->tag = (int32_t)(1 + (r >> 2) % 3); e->val = 0; e->l = l; e->r = rr;
        out.p = e; out.a = a;
    }
    ++*count;
    return out;
}

static uint32_t eval(const Expr *e) {
    switch (e->tag) {
    case 0: return (uint32_t)e->val;
    case 1: return eval(e->l.p) + eval(e->r.p);
    case 2: return eval(e->l.p) * eval(e->r.p);
    default: return eval(e->l.p) - eval(e->r.p);
    }
}

#ifndef M3_NOWALK
/* The end of an `owns Expr using Arena`: Expr's drop (its fields, in order), then the block back
   to the instance it came from, through the kept word ([ARN-2], R3). */
static void drop_expr(ExprBox b) {
    Expr *e = b.p;
    if (e->tag != 0) { drop_expr(e->l); drop_expr(e->r); }
    DEALLOCATE(b.a, e, sizeof(Expr), 8);
}
#endif

int main(void) {
    h_start();
    h_seed(42);
    uint64_t cs = 0;
    Arena arena;
    arena.rt = rt_arena_new((size_t)1 << 20); /* Arena.with_capacity(1 << 20) */
    for (int b = 0; b < BATCHES; b++) {
        /* roots: Array[owns Expr using Arena], from the built-in manager: 16 bytes per element */
        size_t nroots = 0, cap = 0;
        ExprBox *roots = NULL;
        uint64_t count = 0;
        while (count < NB) {
            if (nroots == cap) { cap = cap ? cap * 2 : 4; roots = (ExprBox *)realloc(roots, cap * sizeof(ExprBox)); }
            roots[nroots++] = build(&arena, DEPTH, &count);
        }
        for (size_t i = 0; i < nroots; i++) cs = cs * 1000003u + eval(roots[i].p);
#ifndef M3_NOWALK
        for (size_t i = 0; i < nroots; i++) drop_expr(roots[i]); /* mem.drop(roots): each element ends */
#endif
        free(roots);
        rt_arena_reset(arena.rt); /* arena.reset(): nothing made in it lives ([ALC-5], checked when compiling) */
        cs += count;
    }
    rt_arena_free(arena.rt);
    h_report(NAME, cs);
    return 0;
}
