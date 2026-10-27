/* W2 "lists", plain C with malloc/realloc/free: the C programmer who has no arena. No safety checks.
   The work every program of W2 does: 24 frames of a spatial grid, the classic per-frame list: each
   frame makes 2^18 empty cell lists, puts 2^20 entity ids into random cells (each list grows from
   4 elements by doubling, [HEAP-2]), reads every cell, then ends every list.
   A list is {data, len, cap}, 24 bytes; its buffer is one malloc, grown by realloc, freed at the
   frame's end. The cell array itself is made once and kept (C reuses it).
   Checksum: per cell s = sum of v[k] * (k + 1); cs = cs * 1000003 + s + len. */
#include "harness.h"
#include "m3_rt.h" /* M3_NOINLINE only */

#define G (1u << 18) /* cells */
#define E (1u << 20) /* entities per frame */
#define FRAMES 24

typedef struct List { uint32_t *p; size_t len, cap; } List;
typedef char size_check_List[sizeof(List) == 24 ? 1 : -1]; /* the layout M3.md states */

static M3_NOINLINE void grow(List *l) {
    size_t nc = l->cap ? l->cap * 2 : 4;
    l->p = (uint32_t *)realloc(l->p, nc * sizeof(uint32_t));
    l->cap = nc;
}
static inline void push(List *l, uint32_t v) {
    if (l->len == l->cap) grow(l);
    l->p[l->len++] = v;
}

int main(void) {
    h_start();
    h_seed(42);
    uint64_t cs = 0;
    List *cells = (List *)malloc(G * sizeof(List));
    for (uint32_t f = 0; f < FRAMES; f++) {
        memset(cells, 0, G * sizeof(List));
        for (uint32_t e = 0; e < E; e++) {
            uint32_t r = h_rand();
            push(&cells[r % G], (e * 2654435761u) ^ f);
        }
        for (uint32_t c = 0; c < G; c++) {
            uint64_t s = 0;
            for (size_t k = 0; k < cells[c].len; k++) s += (uint64_t)cells[c].p[k] * (k + 1);
            cs = cs * 1000003u + s + cells[c].len;
        }
        for (uint32_t c = 0; c < G; c++) free(cells[c].p);
    }
    free(cells);
    h_report("plain_lists_malloc", cs);
    return 0;
}
