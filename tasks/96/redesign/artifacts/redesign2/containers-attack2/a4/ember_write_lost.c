/* A4, body that writes the walked list by position, compiled by the first attack's saved rules
   (containers-attack/a4/ember_read_sel.c's form: e in registers; only READS through the list are
   compared with r). The write enemies[0].x += 1.0 goes to the slot, and the end of the step stores
   e over it: the write is lost when enemies[0] is the current element. Not timed: its checksum
   differs from plain_write.c's. */
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
            l.data[slot].x += 1.0f;
            int removed = 0;
            if (e.hp <= 0) { list_push(&l, (Enemy){next, 2 + (int32_t)(h_rand() % 100), 0.0f, 1.0f}); next += 1; removed = 1; }
            if (!removed) { l.data[w] = e; w++; }
            r++;
        }
        l.len = w;
    }
    h_report("a4-write-ember-lost", checksum(&l, next));
    return 0;
}
