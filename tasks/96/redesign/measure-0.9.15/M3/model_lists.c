/* W2 "lists", the model: what Ember emits for this program under spec 0.9.15_Hardened_1 as written.

   The Ember program:

       fn main():
           frame = Arena.with_capacity(1 << 20)
           for f in 0..24:
               cells: Array[Array[u32] using Arena] = Array.new_in(frame)
               cells.reserve(G)
               for c in 0..G:
                   cells.push(Array.new_in(frame))         # an empty list made in `frame`
               for e in 0..E:
                   r = rand()
                   cells[r % G].push((e * 2654435761) ^ f)  # grows in `frame` through the kept word
               for l in cells:
                   s = 0
                   for k, v in l.enumerate(): s += v * (k + 1)
                   cs = cs * 1000003 + s + l.len()
               mem.drop(cells)        # every list ends: each buffer back to `frame` (frees nothing)
               frame.reset()          # allowed: nothing made in `frame` lives ([ALC-5])

   (Ember makes `cells` in the frame arena too, as safe Rust does: a list holding values made in
   `frame` may not outlive the frame's reset, [ALC-5] / E3021, so it is made and ended each frame,
   and in the arena it costs no page faults per frame.)

   What the rule makes it emit, against plain_lists_arena.c:
   - every `Array[T] using Arena` keeps its instance's address (R3, [ALC-1], verdict 47: "a list is
     made in an arena the same way"): {data, len, cap, arena}, 32 bytes against C's 24;
   - growth goes back to the instance through that word ([ALC-2] `allocate` for the first block,
     `reallocate` after), with [HEAP-8]'s capacity check and [ALC-4]'s null check;
   - each list's end gives its buffer back through the word ([ARN-2]: a no-op);
   - `cells[r % G]` is bounds-checked (today's check, not this rework's; kept in every Ember form
     here, the salvages included, so it does not blur the word's cost; plain C has none).

   Two forms, one source: default "model_lists" (Arena's operations are runtime calls in another
   translation unit, m3_rt.c, as today); -DM3_INLINE "model_lists_inl" (compiled into the program). */
#include "harness.h"
#include "m3_rt.h"

#define G (1u << 18)
#define E (1u << 20)
#define FRAMES 24

typedef struct Arena { RtArena *rt; } Arena; /* Ember's `Arena` value; the word is its address */

#ifdef M3_INLINE
#define ALLOCATE(inst, size, align) rt_arena_bump((inst)->rt, (size), (align))
#define REALLOCATE(inst, p, old, nw, align) rt_arena_grow_inl((inst)->rt, (p), (old), (nw), (align))
static inline void deallocate_inl(Arena *inst, void *p, size_t size, size_t align) {
    (void)inst; (void)p; (void)size; (void)align; /* [ARN-2] */
}
#define DEALLOCATE(inst, p, size, align) deallocate_inl((inst), (p), (size), (align))
#define NAME "model_lists_inl"
#else
#define ALLOCATE(inst, size, align) rt_arena_alloc_uninit((inst)->rt, 1, (size), (align))
#define REALLOCATE(inst, p, old, nw, align) rt_arena_reallocate((inst)->rt, (p), (old), (nw), (align))
#define DEALLOCATE(inst, p, size, align) rt_arena_deallocate((inst)->rt, (p), (size), (align))
#define NAME "model_lists"
#endif

/* `Array[u32] using Arena`: 32 bytes (R3). */
typedef struct List { uint32_t *p; size_t len, cap; Arena *a; } List;
typedef char size_check_List[sizeof(List) == 32 ? 1 : -1]; /* the layout M3.md states */
/* `Array[Array[u32] using Arena] using Arena`: the cell list, the same four words. */
typedef struct Cells { List *p; size_t len, cap; Arena *a; } Cells;

static M3_NOINLINE void grow(List *l) {
    size_t nc = l->cap ? l->cap * 2 : 4;                                   /* [HEAP-2] */
    if (nc > (size_t)PTRDIFF_MAX / sizeof(uint32_t)) m3_panic("capacity overflow"); /* [HEAP-8] */
    void *q = l->p == NULL
        ? ALLOCATE(l->a, nc * sizeof(uint32_t), 4)                        /* through the word (R3) */
        : REALLOCATE(l->a, l->p, l->cap * sizeof(uint32_t), nc * sizeof(uint32_t), 4);
    if (q == NULL) m3_panic("out of memory");                              /* [ALC-4] */
    l->p = (uint32_t *)q;
    l->cap = nc;
}
static inline void push(List *l, uint32_t v) {
    if (l->len == l->cap) grow(l);
    l->p[l->len++] = v;
}

static M3_NOINLINE void cells_reserve(Cells *c, size_t n) { /* cells.reserve(n) on an empty list */
    if (n > (size_t)PTRDIFF_MAX / sizeof(List)) m3_panic("capacity overflow");  /* [HEAP-8] */
    void *q = ALLOCATE(c->a, n * sizeof(List), 8);
    if (q == NULL) m3_panic("out of memory");                                    /* [ALC-4] */
    c->p = (List *)q;
    c->cap = n;
}
static M3_NOINLINE void cells_grow(Cells *c) {
    size_t nc = c->cap ? c->cap * 2 : 4;
    if (nc > (size_t)PTRDIFF_MAX / sizeof(List)) m3_panic("capacity overflow");
    void *q = c->p == NULL ? ALLOCATE(c->a, nc * sizeof(List), 8)
                           : REALLOCATE(c->a, c->p, c->cap * sizeof(List), nc * sizeof(List), 8);
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
        Cells cells = { NULL, 0, 0, &frame };      /* Array.new_in(frame): the word kept (R3) */
        cells_reserve(&cells, G);
        for (uint32_t c = 0; c < G; c++) {
            if (cells.len == cells.cap) cells_grow(&cells);
            List empty = { NULL, 0, 0, &frame };     /* Array.new_in(frame) */
            cells.p[cells.len++] = empty;
        }
        for (uint32_t e = 0; e < E; e++) {
            uint32_t r = h_rand();
            size_t i = r % G;
            if (i >= cells.len) m3_panic("index out of bounds"); /* bounds check (today's) */
            push(&cells.p[i], (e * 2654435761u) ^ f);
        }
        for (size_t c = 0; c < cells.len; c++) {
            const List *l = &cells.p[c];
            uint64_t s = 0;
            for (size_t k = 0; k < l->len; k++) s += (uint64_t)l->p[k] * (k + 1);
            cs = cs * 1000003u + s + l->len;
        }
        /* mem.drop(cells): each list's end, its buffer back through its word ([ARN-2], R3) */
        for (size_t c = 0; c < cells.len; c++)
            if (cells.p[c].p != NULL) DEALLOCATE(cells.p[c].a, cells.p[c].p, cells.p[c].cap * sizeof(uint32_t), 4);
        DEALLOCATE(cells.a, cells.p, cells.cap * sizeof(List), 8);
        rt_arena_reset(frame.rt); /* frame.reset(): nothing made in it lives ([ALC-5]) */
    }
    rt_arena_free(frame.rt);
    h_report(NAME, cs);
    return 0;
}
