/* Plain C baseline: the fastest one-pass order-keeping way found. An in-place filter with a read
   position r and a write position w over the live length; each element is loaded once into a local,
   updated there, and stored once at its final place w (so no load ever waits on a store to the same
   element). No checks. */
#include "common.h"
int main(void) {
    h_start();
    List l; fill(&l);
    uint32_t next = N;
    for (int f = 0; f < FRAMES; f++) {
        size_t r = 0, w = 0;
        while (r < l.len) {
            Enemy v = l.data[r];
            v.x += v.vx; v.hp -= 1;
            if (v.hp <= 0) {
                list_push(&l, (Enemy){next++, 2 + (int32_t)(h_rand() % 100), 0.0f, 1.0f});
                r++; continue;                       /* removed: not stored */
            }
            l.data[w] = v;
            w++; r++;
        }
        l.len = w;
    }
    h_report("a4-plain", checksum(&l, next));
    return 0;
}
