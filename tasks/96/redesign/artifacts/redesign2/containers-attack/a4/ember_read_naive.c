/* A4 timing, the answer AS WRITTEN for a body that reads the walked list by position (plain_read.c has
   the workload). The report keeps e in registers only when "nothing but e touches enemies[r]"; a read
   of enemies[k] may touch it, so e is the element in its slot (A2's name): the body updates the slot,
   and the end of the step copies the slot down to w. That is the update-then-copy form of
   phase3b-containers.md section 1.4. */
#include "../../containers/bench/a4/common.h"
int main(void) {
    h_start();
    List l; fill(&l);
    uint32_t next = N; uint64_t lead = 0;
    for (int f = 0; f < FRAMES; f++) {
        size_t r = 0, w = 0;
        while (r < l.len) {
            Enemy *e = &l.data[r];
            e->x += e->vx; e->hp -= 1;
            size_t k = 0, slot = k < w ? k : k + (r - w);
            lead += (uint32_t)l.data[slot].hp;
            if (l.data[r].hp <= 0) { list_push(&l, (Enemy){next, 2 + (int32_t)(h_rand() % 100), 0.0f, 1.0f}); next += 1; r++; continue; }
            if (w != r) l.data[w] = l.data[r];
            w++; r++;
        }
        l.len = w;
    }
    h_report("a4-read-ember-naive", checksum(&l, next) + lead);
    return 0;
}
