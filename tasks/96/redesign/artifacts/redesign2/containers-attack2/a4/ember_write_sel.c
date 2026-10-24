/* A4, body that writes the walked list by position, SAVED: a write through the list compares the
   mapped position with r and writes e (in registers) when they are equal, the slot otherwise: the
   same compare as the first attack's read rule (containers-attack/a4/ember_read_sel.c). */
#include "../../containers/bench/a4/common.h"
int main(void) {
    h_start();
    List l; fill(&l);
    uint32_t next = N;
    for (int f = 0; f < FRAMES; f++) {
        size_t r = 0, w = 0;
        while (r < l.len) {
            Enemy e = l.data[r];
            e.x += e.vx; e.hp -= 1;
            size_t k = 0, slot = k < w ? k : k + (r - w);    /* enemies[0] */
            if (slot == r) e.x += 1.0f; else l.data[slot].x += 1.0f;
            int removed = 0;
            if (e.hp <= 0) { list_push(&l, (Enemy){next, 2 + (int32_t)(h_rand() % 100), 0.0f, 1.0f}); next += 1; removed = 1; }
            if (!removed) { l.data[w] = e; w++; }
            r++;
        }
        l.len = w;
    }
    h_report("a4-write-ember-sel", checksum(&l, next));
    return 0;
}
