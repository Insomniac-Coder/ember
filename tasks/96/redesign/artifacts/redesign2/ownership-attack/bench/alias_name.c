/* alias_name.c: the cost of the fix for "a name keeps the old value when a write that may be to the
 * same place replaces it" (names follow values, rule 2, when the compiler cannot tell the two places
 * apart: `old = p.name; q.name = new; use(old)` with p and q possibly one object, or
 * `n = xs[i]; xs[j] = v; use(n)` with i and j possibly equal).
 *
 *   -DMODE=0  no compare: the write drops the old value as if it were not the named one (what C code
 *             that forgot the aliasing case does; when the two are the same, `old` reads the NEW
 *             value, which is wrong, so this model prints a different checksum)
 *   -DMODE=1  the fix: one pointer compare at the write; when equal the old value moves into the
 *             name's own storage instead of being dropped, and is dropped after the name's last use
 *   -DMODE=2  Rust's spelling of the same program: copy the field before the write (`.clone()`)
 *   -DMODE=3  the fix as the compiler would rather emit it: the name holds the value's bits in a register
 *             (valid because nothing in the name's life changes the value in place, which the effect
 *             summaries tell); the write compares the two places, and the old value is dropped either
 *             at the write (different places) or after the name's last use (the same place)
 *
 * 4,096 objects, each owning a heap block (its "name"); 2^27 steps; each step names the field of a
 * random object, replaces the field of another random object (the same one in 1 step of 4,096), and
 * reads the name. Blocks come from a free list so that the allocator does not hide the compare.
 * Build: clang -O2 -DMODE=n alias_name.c -o alias_name_n.exe */
#include "../../../redesign/bench/harness.h"
#ifndef MODE
#define MODE 1
#endif
#define N 4096u
#define STEPS (1u << 27)
typedef struct Blk { uint64_t v; struct Blk *next; } Blk;
typedef struct { Blk *name; uint64_t pad; } Obj;
static Blk *free_list;
static inline Blk *blk_new(uint64_t v) { Blk *b = free_list; free_list = b->next; b->v = v; return b; }
static inline void blk_drop(Blk *b) { b->next = free_list; free_list = b; }
int main(void) {
    h_start();
    h_seed(7);
    Obj *objs = (Obj *)malloc(N * sizeof(Obj));
    Blk *pool = (Blk *)malloc((2 * N + 8) * sizeof(Blk));
    if (!objs || !pool) abort();
    for (uint32_t k = 0; k < 2 * N + 8; k++) blk_drop(&pool[k]);
    for (uint32_t k = 0; k < N; k++) objs[k].name = blk_new(k);
    uint64_t acc = 0;
    for (uint32_t s = 0; s < STEPS; s++) {
        uint32_t r = h_rand();
        uint32_t a = r & (N - 1), b = (r >> 12) & (N - 1);
#if MODE == 3
        Blk **oldp = &objs[a].name;                            /* old = p.name */
        Blk *oldv = *oldp;
        Blk *nb = blk_new(s);
        Blk *prev = objs[b].name;
        objs[b].name = nb;                                     /* q.name = new */
        int keep = (&objs[b].name == oldp);                    /* the one compare */
        if (!keep) blk_drop(prev);
        acc += oldv->v * 3 + (oldv->v >> 7);                   /* use(old) */
        if (keep) blk_drop(oldv);                              /* the name's last use */
#elif MODE == 2
        Blk *copy = blk_new(objs[a].name->v);                 /* old = p.name.clone() */
        Blk *nb = blk_new(s);
        blk_drop(objs[b].name); objs[b].name = nb;             /* q.name = new */
        acc += copy->v * 3 + (copy->v >> 7);                   /* use(old) */
        blk_drop(copy);
#else
        Blk **old = &objs[a].name;                             /* old = p.name: a name, no copy */
        Blk *nb = blk_new(s);
#if MODE == 1
        Blk *own = NULL;
        if (&objs[b].name == old) { own = objs[b].name; old = &own; }   /* the fix: keep the old value */
        else blk_drop(objs[b].name);
#else
        blk_drop(objs[b].name);
#endif
        objs[b].name = nb;                                     /* q.name = new */
        acc += (*old)->v * 3 + ((*old)->v >> 7);               /* use(old) */
#if MODE == 1
        if (own) blk_drop(own);                                /* the name's last use */
#endif
#endif
    }
    for (uint32_t k = 0; k < N; k++) acc += objs[k].name->v;
#if MODE == 0
    h_report("alias-name-no-compare", acc);
#elif MODE == 1
    h_report("alias-name-compare", acc);
#elif MODE == 3
    h_report("alias-name-compare-reg", acc);
#else
    h_report("alias-name-copy-first", acc);
#endif
    return 0;
}
