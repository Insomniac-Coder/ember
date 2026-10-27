/* W1 "trees", salvage (a) of verdict 47: an arena kind that never frees single values.

   The idea: the kind, not the instance, says that giving a block back does nothing. Then
   - a fixed-size value made in it (`owns T using A`, a class object `using a`) never needs its
     instance again: it does not grow, and its end gives nothing back. The word goes: an
     `owns Expr using Arena` place is the block's address alone, 8 bytes (an inner node 24 bytes,
     not 40);
   - its end runs only T's own drop. Expr has none (an int, and links into the same no-free arena),
     so a tree's end does nothing at all: no walk ([ARN-2]'s "frees nothing one by one", taken at
     its word).
   Everything else stays as the model emits it: `arena.allocate` for each node, here with the bump
   compiled into the program (model_trees_inl's form), and the null test ([ALC-4]).

   What it needs from the language: the `Allocator` kind says so where it is declared, checked
   when compiling, e.g. `unsafe extend Arena implements Allocator(frees_nothing):` whose
   `deallocate` must then be empty (or absent); the compiler's "needs drop" test then counts an
   `owns T using A` of such a kind as needing drop only when T does. A kind that frees (a pool)
   keeps the word, or uses way (b).

   Safety kept, exactly: (1) a value made in `arena` still lives no longer than `arena` (E3063,
   [LT-4]): that check reads the making line (`using arena`), never the stored word; (2) `reset`
   and the arena's end are still refused while such a value lives ([ALC-5], E3021), so no block is
   reused under a live value; (3) a T that needs drop still has its drop run at its end; only the
   call that gave nothing back goes. Growth is not covered: a list made in such an arena still
   needs its instance when it grows (W2's salvages). */
#include "harness.h"
#include "m3_rt.h"

#define NB (1u << 20)
#define BATCHES 16
#define DEPTH 12

typedef struct Arena { RtArena *rt; } Arena;
typedef struct Expr Expr;
struct Expr { int32_t tag, val; Expr *l, *r; }; /* `owns Expr using Arena` (frees nothing): 8 bytes */
typedef char size_check_Expr[sizeof(Expr) == 24 ? 1 : -1]; /* the layout M3.md states */

static Expr *new_expr(Arena *a) {
    Expr *p = (Expr *)rt_arena_bump(a->rt, sizeof(Expr), 8); /* arena.allocate, inline */
    if (p == NULL) m3_panic("out of memory");                 /* [ALC-4] */
    return p;
}

static Expr *build(Arena *a, int d, uint64_t *count) {
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

int main(void) {
    h_start();
    h_seed(42);
    uint64_t cs = 0;
    Arena arena;
    arena.rt = rt_arena_new((size_t)1 << 20);
    for (int b = 0; b < BATCHES; b++) {
        size_t nroots = 0, cap = 0;
        Expr **roots = NULL; /* Array[owns Expr using Arena]: 8 bytes per element */
        uint64_t count = 0;
        while (count < NB) {
            if (nroots == cap) { cap = cap ? cap * 2 : 4; roots = (Expr **)realloc(roots, cap * sizeof(Expr *)); }
            roots[nroots++] = build(&arena, DEPTH, &count);
        }
        for (size_t i = 0; i < nroots; i++) cs = cs * 1000003u + eval(roots[i]);
        free(roots); /* mem.drop(roots): the elements need no drop, so nothing runs per tree */
        rt_arena_reset(arena.rt);
        cs += count;
    }
    rt_arena_free(arena.rt);
    h_report("salvage_trees_nofree", cs);
    return 0;
}
