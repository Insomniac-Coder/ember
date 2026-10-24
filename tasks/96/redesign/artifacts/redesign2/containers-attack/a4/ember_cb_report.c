/* A4 timing, unseen call in the body (plain_cb.c has the workload): the answer AS THE REPORT MODELS IT
   (ember_swapfill.c) plus what rule 9 adds: the walkers count in the top byte of the capacity word
   (raised at the loop's start, lowered at its end; push masks it off), and rule 10's re-checks
   r < len and w <= r after the call. e stays in registers across the call. Elements are plain data
   standing in for elements that own memory, as in the report's ember_swapfill.c. */
#include "../../containers/bench/a4/common.h"
#define ONE (1ull << 56)
#define MASK (ONE - 1)
static void panic(const char *m) { fprintf(stderr, "panic: %s\n", m); exit(101); }
static void grow_masked(List *l) { size_t c = l->cap & MASK; c = c ? c * 2 : 4; l->data = realloc(l->data, c * sizeof(Enemy)); l->cap = (l->cap & ~MASK) | c; }
static inline void push_masked(List *l, Enemy e) { if (l->len == (l->cap & MASK)) grow_masked(l); l->data[l->len++] = e; }
static volatile uint64_t sink;
static void on_step(List *l) { sink += l->len; }
static void (*volatile cb)(List *) = on_step;
int main(void) {
    h_start();
    List l; fill(&l);
    uint32_t next = N;
    for (int f = 0; f < FRAMES; f++) {
        l.cap += ONE;
        size_t r = 0, w = 0;
        while (r < l.len) {
            Enemy e = l.data[r];
            e.x += e.vx; e.hp -= 1;
            cb(&l);
            if (__builtin_expect(!(r < l.len && w <= r), 0)) panic("list changed under the loop");
            int removed = 0;
            if (e.hp <= 0) { push_masked(&l, (Enemy){next, 2 + (int32_t)(h_rand() % 100), 0.0f, 1.0f}); next += 1; removed = 1; }
            if (!removed) { if (w != r) l.data[r] = l.data[w]; l.data[w] = e; w++; }
            r++;
        }
        l.len = w;
        l.cap -= ONE;
    }
    h_report("a4-cb-ember-report", checksum(&l, next));
    return 0;
}
