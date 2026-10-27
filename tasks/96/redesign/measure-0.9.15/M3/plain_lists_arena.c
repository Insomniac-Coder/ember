/* W2 "lists", plain C with an arena: what a careful C programmer writes for per-frame lists.
   No safety checks. Same work as plain_lists_malloc.c (24 frames; 2^18 cell lists made empty, 2^20
   pushes into random cells, every cell read, every list ended), with every buffer and the cell
   array itself taken from one frame arena (m3_rt.h's RtArena, inline bump) and the frame ended by
   one reset. A list is {data, len, cap}, 24 bytes: the C programmer passes the arena where a list
   grows (`push(&cells[i], frame, v)`), so no list keeps it. Growth is in place when the buffer is
   the arena's last block, else a new block and a copy (the old block stays until the reset). */
#include "harness.h"
#include "m3_rt.h"

#define G (1u << 18)
#define E (1u << 20)
#define FRAMES 24

typedef struct List { uint32_t *p; size_t len, cap; } List;
typedef char size_check_List[sizeof(List) == 24 ? 1 : -1]; /* the layout M3.md states */

static M3_NOINLINE void grow(List *l, RtArena *a) {
    size_t nc = l->cap ? l->cap * 2 : 4;
    l->p = (uint32_t *)rt_arena_grow_inl(a, l->p, l->cap * sizeof(uint32_t), nc * sizeof(uint32_t), 4);
    l->cap = nc;
}
static inline void push(List *l, RtArena *a, uint32_t v) {
    if (l->len == l->cap) grow(l, a);
    l->p[l->len++] = v;
}

int main(void) {
    h_start();
    h_seed(42);
    uint64_t cs = 0;
    RtArena *frame = rt_arena_new((size_t)1 << 20);
    for (uint32_t f = 0; f < FRAMES; f++) {
        List *cells = (List *)rt_arena_bump(frame, G * sizeof(List), 8);
        memset(cells, 0, G * sizeof(List));
        for (uint32_t e = 0; e < E; e++) {
            uint32_t r = h_rand();
            push(&cells[r % G], frame, (e * 2654435761u) ^ f);
        }
        for (uint32_t c = 0; c < G; c++) {
            uint64_t s = 0;
            for (size_t k = 0; k < cells[c].len; k++) s += (uint64_t)cells[c].p[k] * (k + 1);
            cs = cs * 1000003u + s + cells[c].len;
        }
        rt_arena_reset(frame);
    }
    rt_arena_free(frame);
    h_report("plain_lists_arena", cs);
    return 0;
}
