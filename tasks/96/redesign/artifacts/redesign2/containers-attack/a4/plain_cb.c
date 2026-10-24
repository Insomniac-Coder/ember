/* A4 timing, body that calls code the compiler cannot see (a callback through a function pointer)
   which reads the list's length. Workload ../../containers/bench/a4/common.h; the callback runs once
   per step, after the update and before the removal test.
       for e in w.enemies:
           e.x += e.vx
           e.hp -= 1
           w.on_step()            # unseen: sink += len(self.enemies)
           if e.hp <= 0: w.enemies.push(...); w.enemies.remove(e)
   Plain C, no checks: element in registers across the call (in C the callback does not see the
   update); the list header is read again after the call, since its address escaped. */
#include "../../containers/bench/a4/common.h"
static volatile uint64_t sink;
static void on_step(List *l) { sink += l->len; }
static void (*volatile cb)(List *) = on_step;
int main(void) {
    h_start();
    List l; fill(&l);
    uint32_t next = N;
    for (int f = 0; f < FRAMES; f++) {
        size_t r = 0, w = 0;
        while (r < l.len) {
            Enemy v = l.data[r];
            v.x += v.vx; v.hp -= 1;
            cb(&l);
            if (v.hp <= 0) { list_push(&l, (Enemy){next++, 2 + (int32_t)(h_rand() % 100), 0.0f, 1.0f}); r++; continue; }
            l.data[w] = v; w++; r++;
        }
        l.len = w;
    }
    h_report("a4-cb-plain", checksum(&l, next));
    return 0;
}
