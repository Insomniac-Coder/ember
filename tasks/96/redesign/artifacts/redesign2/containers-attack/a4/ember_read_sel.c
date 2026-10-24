/* A4 timing, the SAVED answer, second form, for a body that reads the walked list by position
   (plain_read.c has the workload). A read of a field through the list compares the mapped position
   with r and takes the field from e (in registers) when they are equal: a compare and a select, no
   store. A write-back to the slot is kept only for code that takes the whole element by reference or
   that the compiler cannot see. */
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
            size_t k = 0, slot = k < w ? k : k + (r - w);    /* enemies[0] */
            lead += (uint32_t)(slot == r ? e.hp : l.data[slot].hp);
            int removed = 0;
            if (e.hp <= 0) { list_push(&l, (Enemy){next, 2 + (int32_t)(h_rand() % 100), 0.0f, 1.0f}); next += 1; removed = 1; }
            if (!removed) { l.data[w] = e; w++; }
            r++;
        }
        l.len = w;
    }
    h_report("a4-read-ember-sel", checksum(&l, next) + lead);
    return 0;
}
