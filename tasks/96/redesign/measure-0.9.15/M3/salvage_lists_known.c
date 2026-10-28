/* W2 "lists", salvage (a+) of verdict 47: the instance known where the list grows, so the list
   keeps no word. Way (a) (an arena kind that never frees single values) drops the word for values
   that never grow; a list grows, so (a) alone keeps the word for lists. This salvage removes it for
   lists too, the way the C programmer does in plain_lists_arena.c: the instance is passed where a
   list grows, not stored in it.

   What it needs from the language: each growth site must know, when compiling, which instance the
   list came from. That holds when the place's type names the instance, not only the kind:
   `cells: Array[Array[u32] using frame] using frame`, where `frame` is a name in scope (a type
   that names a variable, as a Rust lifetime brand does). A list given to a function then needs
   the instance given too, or the function's parameter names it (`fn add(ref mut l: Array[u32]
   using a, ref mut a: Arena)`), which is more to write and read than verdict 47's form; a list
   whose instance is not known where it grows (kept in a struct used across functions without the
   instance) keeps the word. With (a)'s no-free kind its end does nothing; with another kind its
   end needs the instance where it ends, the same way.

   Safety kept, exactly: the instance passed is the one named in the list's type, checked when
   compiling, so growth goes back to the instance the list came from (what the word was for, R3);
   the lifetime check (E3063) and [ALC-5] are unchanged. Every other check of model_lists_inl stays
   ([HEAP-8], [ALC-4], the bounds check); the arena is the same, inline. */
#include "harness.h"
#include "m3_rt.h"

#define G (1u << 18)
#define E (1u << 20)
#define FRAMES 24

typedef struct Arena { RtArena *rt; } Arena;
typedef struct List { uint32_t *p; size_t len, cap; } List; /* `Array[u32] using frame`: 24 bytes */
typedef char size_check_List[sizeof(List) == 24 ? 1 : -1]; /* the layout M3.md states */
typedef struct Cells { List *p; size_t len, cap; } Cells;

static M3_NOINLINE void grow(List *l, Arena *a) { /* `a`: the instance the list's type names */
    size_t nc = l->cap ? l->cap * 2 : 4;                                   /* [HEAP-2] */
    if (nc > (size_t)PTRDIFF_MAX / sizeof(uint32_t)) m3_panic("capacity overflow"); /* [HEAP-8] */
    void *q = l->p == NULL
        ? rt_arena_bump(a->rt, nc * sizeof(uint32_t), 4)
        : rt_arena_grow_inl(a->rt, l->p, l->cap * sizeof(uint32_t), nc * sizeof(uint32_t), 4);
    if (q == NULL) m3_panic("out of memory");                              /* [ALC-4] */
    l->p = (uint32_t *)q;
    l->cap = nc;
}
static inline void push(List *l, Arena *a, uint32_t v) {
    if (l->len == l->cap) grow(l, a);
    l->p[l->len++] = v;
}

static M3_NOINLINE void cells_reserve(Cells *c, Arena *a, size_t n) {
    if (n > (size_t)PTRDIFF_MAX / sizeof(List)) m3_panic("capacity overflow");
    void *q = rt_arena_bump(a->rt, n * sizeof(List), 8);
    if (q == NULL) m3_panic("out of memory");
    c->p = (List *)q;
    c->cap = n;
}
static M3_NOINLINE void cells_grow(Cells *c, Arena *a) {
    size_t nc = c->cap ? c->cap * 2 : 4;
    if (nc > (size_t)PTRDIFF_MAX / sizeof(List)) m3_panic("capacity overflow");
    void *q = c->p == NULL ? rt_arena_bump(a->rt, nc * sizeof(List), 8)
                           : rt_arena_grow_inl(a->rt, c->p, c->cap * sizeof(List), nc * sizeof(List), 8);
    if (q == NULL) m3_panic("out of memory");
    c->p = (List *)q;
    c->cap = nc;
}

int main(void) {
    h_start();
    h_seed(42);
    uint64_t cs = 0;
    Arena frame;
    frame.rt = rt_arena_new((size_t)1 << 20);
    for (uint32_t f = 0; f < FRAMES; f++) {
        Cells cells = { NULL, 0, 0 };
        cells_reserve(&cells, &frame, G);
        for (uint32_t c = 0; c < G; c++) {
            if (cells.len == cells.cap) cells_grow(&cells, &frame);
            List empty = { NULL, 0, 0 };
            cells.p[cells.len++] = empty;
        }
        for (uint32_t e = 0; e < E; e++) {
            uint32_t r = h_rand();
            size_t i = r % G;
            if (i >= cells.len) m3_panic("index out of bounds"); /* bounds check (today's) */
            push(&cells.p[i], &frame, (e * 2654435761u) ^ f);
        }
        for (size_t c = 0; c < cells.len; c++) {
            const List *l = &cells.p[c];
            uint64_t s = 0;
            for (size_t k = 0; k < l->len; k++) s += (uint64_t)l->p[k] * (k + 1);
            cs = cs * 1000003u + s + l->len;
        }
        /* mem.drop(cells): the arena frees nothing one by one ([ARN-2]), so nothing runs per list */
        rt_arena_reset(frame.rt);
    }
    rt_arena_free(frame.rt);
    h_report("salvage_lists_known", cs);
    return 0;
}
