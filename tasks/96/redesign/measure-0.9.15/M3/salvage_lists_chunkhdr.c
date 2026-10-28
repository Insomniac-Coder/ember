/* W2 "lists", salvage (b) of verdict 47 for growing lists: the buffer's address finds its arena.

   The arena (m3_rt.h's HArena) takes its chunks on 64 KB boundaries and writes its own address in
   each chunk's first 64 bytes; a list's instance is `((HChunk *)(data & ~0xFFFF))->arena`, one AND
   and one load, done only when the list grows (and when it ends, where `Arena`'s `deallocate` does
   nothing, so after inlining nothing). The list keeps no word: {data, len, cap}, 24 bytes.

   The one new piece: a list with no buffer yet still needs an address that finds its arena, so
   `Array.new_in(frame)` gives it a zero-size block, a real address inside one of the arena's chunks
   (its first data byte; h_arena_empty) that is never read. No memory, no extra word; a list made
   empty in the built-in manager keeps today's null.

   What it needs from the language: as salvage_trees_chunkhdr.c (one more unsafe promise an
   `Allocator` kind may make: "my blocks, zero-size ones included, find their instance"), nothing in
   the program's text. Lists whose kind does not make the promise keep the word.

   Safety kept, exactly: growth goes back to the instance the list's buffer came from, by the kind's
   promise; E3063 and [ALC-5] read the making line, not the word, so they are unchanged. Every other
   check of model_lists_inl stays ([HEAP-8], [ALC-4], the bounds check). Cost: 64 bytes per 64 KB. */
#include "harness.h"
#include "m3_rt.h"

#define G (1u << 18)
#define E (1u << 20)
#define FRAMES 24

typedef struct List { uint32_t *p; size_t len, cap; } List; /* `Array[u32] using Arena`: 24 bytes */
typedef char size_check_List[sizeof(List) == 24 ? 1 : -1]; /* the layout M3.md states */
typedef struct Cells { List *p; size_t len, cap; } Cells;

static inline void deallocate_inl(HArena *inst, void *p, size_t size, size_t align) {
    (void)inst; (void)p; (void)size; (void)align; /* [ARN-2] */
}

static M3_NOINLINE void grow(List *l) {
    size_t nc = l->cap ? l->cap * 2 : 4;                                   /* [HEAP-2] */
    if (nc > (size_t)PTRDIFF_MAX / sizeof(uint32_t)) m3_panic("capacity overflow"); /* [HEAP-8] */
    HArena *a = h_arena_of(l->p);                                          /* the instance, from the address */
    void *q = h_arena_grow_inl(a, l->p, l->cap * sizeof(uint32_t), nc * sizeof(uint32_t), 4);
    if (q == NULL) m3_panic("out of memory");                              /* [ALC-4] */
    l->p = (uint32_t *)q;
    l->cap = nc;
}
static inline void push(List *l, uint32_t v) {
    if (l->len == l->cap) grow(l);
    l->p[l->len++] = v;
}

static M3_NOINLINE void cells_grow(Cells *c) {
    size_t nc = c->cap ? c->cap * 2 : 4;
    if (nc > (size_t)PTRDIFF_MAX / sizeof(List)) m3_panic("capacity overflow");
    void *q = h_arena_grow_inl(h_arena_of(c->p), c->p, c->cap * sizeof(List), nc * sizeof(List), 8);
    if (q == NULL) m3_panic("out of memory");
    c->p = (List *)q;
    c->cap = nc;
}

int main(void) {
    h_start();
    h_seed(42);
    uint64_t cs = 0;
    HArena *frame = h_arena_new();
    for (uint32_t f = 0; f < FRAMES; f++) {
        Cells cells = { (List *)h_arena_empty(frame), 0, 0 }; /* Array.new_in(frame) */
        {                                                     /* cells.reserve(G) */
            if ((size_t)G > (size_t)PTRDIFF_MAX / sizeof(List)) m3_panic("capacity overflow");
            void *q = h_arena_bump(h_arena_of(cells.p), G * sizeof(List), 8);
            if (q == NULL) m3_panic("out of memory");
            cells.p = (List *)q;
            cells.cap = G;
        }
        for (uint32_t c = 0; c < G; c++) {
            if (cells.len == cells.cap) cells_grow(&cells);
            List empty = { (uint32_t *)h_arena_empty(frame), 0, 0 }; /* Array.new_in(frame) */
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
        /* mem.drop(cells): each buffer back to the instance its address finds ([ARN-2], R3) */
        for (size_t c = 0; c < cells.len; c++)
            deallocate_inl(h_arena_of(cells.p[c].p), cells.p[c].p, cells.p[c].cap * sizeof(uint32_t), 4);
        deallocate_inl(h_arena_of(cells.p), cells.p, cells.cap * sizeof(List), 8);
        h_arena_reset(frame);
    }
    h_arena_free(frame);
    h_report("salvage_lists_chunkhdr", cs);
    return 0;
}
