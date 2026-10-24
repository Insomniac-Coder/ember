/* A4 timing, body that also reads the list by position. Workload: ../../containers/bench/a4/common.h,
   plus each step reads enemies[0].hp (the leader) and adds it to the checksum.
       for e in enemies:
           e.x += e.vx
           e.hp -= 1
           lead += enemies[0].hp
           if e.hp <= 0: enemies.push(...); enemies.remove(e)
   Plain C, fastest found: element in registers; enemies[0] is the current element when nothing has
   been kept yet in this frame (w == 0), so the hand-written code reads it from the registers then. */
#include "../../containers/bench/a4/common.h"
int main(void) {
    h_start();
    List l; fill(&l);
    uint32_t next = N; uint64_t lead = 0;
    for (int f = 0; f < FRAMES; f++) {
        size_t r = 0, w = 0;
        while (r < l.len) {
            Enemy v = l.data[r];
            v.x += v.vx; v.hp -= 1;
            lead += (uint32_t)(w == 0 ? v.hp : l.data[0].hp);
            if (v.hp <= 0) { list_push(&l, (Enemy){next++, 2 + (int32_t)(h_rand() % 100), 0.0f, 1.0f}); r++; continue; }
            l.data[w] = v; w++; r++;
        }
        l.len = w;
    }
    h_report("a4-read-plain", checksum(&l, next) + lead);
    return 0;
}
