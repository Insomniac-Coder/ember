/* A4 timing, body that WRITES the walked list by position (the first attack timed reads only:
   containers-attack/a4/plain_read.c). Workload ../../containers/bench/a4/common.h, plus each step
   moves the leader: enemies[0].x += 1.0 (Python: probes/py_attack2.py check 3).
       for e in enemies:
           e.x += e.vx
           e.hp -= 1
           enemies[0].x += 1.0
           if e.hp <= 0: enemies.push(...); enemies.remove(e)
   Plain C, fastest found: element in registers; enemies[0] is the current element while nothing has
   been kept in this frame (w == 0), so the hand-written code writes the register copy then. */
#include "../../containers/bench/a4/common.h"
int main(void) {
    h_start();
    List l; fill(&l);
    uint32_t next = N;
    for (int f = 0; f < FRAMES; f++) {
        size_t r = 0, w = 0;
        while (r < l.len) {
            Enemy v = l.data[r];
            v.x += v.vx; v.hp -= 1;
            if (w == 0) v.x += 1.0f; else l.data[0].x += 1.0f;
            if (v.hp <= 0) { list_push(&l, (Enemy){next++, 2 + (int32_t)(h_rand() % 100), 0.0f, 1.0f}); r++; continue; }
            l.data[w] = v; w++; r++;
        }
        l.len = w;
    }
    h_report("a4-write-plain", checksum(&l, next));
    return 0;
}
