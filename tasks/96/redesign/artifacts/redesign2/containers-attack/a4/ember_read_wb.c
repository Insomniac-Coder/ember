/* A4 timing, the SAVED answer for a body that reads the walked list by position (plain_read.c has the
   workload). e stays in registers; before the body reads the list by position, e is stored back to its
   slot r (one 16-byte store), so the read sees the updated element; the read goes through rule 4's
   mapping (k < w ? k : k + (r - w)). At the end of the step e is stored at w as before. */
#include "../../containers/bench/a4/common.h"
int main(void) {
    h_start();
    List l; fill(&l);
    uint32_t next = N; uint64_t lead = 0;
    for (int f = 0; f < FRAMES; f++) {
        size_t r = 0, w = 0;
        while (r < l.len) {
            Enemy e = l.data[r];
            e.x += e.vx; e.hp -= 1;
            l.data[r] = e;                                   /* write-back before a read by position */
            size_t k = 0, slot = k < w ? k : k + (r - w);    /* enemies[0] */
            lead += (uint32_t)l.data[slot].hp;
            int removed = 0;
            if (e.hp <= 0) { list_push(&l, (Enemy){next, 2 + (int32_t)(h_rand() % 100), 0.0f, 1.0f}); next += 1; removed = 1; }
            if (!removed) { l.data[w] = e; w++; }
            r++;
        }
        l.len = w;
    }
    h_report("a4-read-ember-wb", checksum(&l, next) + lead);
    return 0;
}
