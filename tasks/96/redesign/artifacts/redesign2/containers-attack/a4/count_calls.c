/* Not timed: counts the callback calls of the plain_cb.c workload (one per step), so a time difference
   can be divided per call. */
#include "../../containers/bench/a4/common.h"
static uint64_t calls;
int main(void) {
    h_start();
    List l; fill(&l);
    uint32_t next = N;
    for (int f = 0; f < FRAMES; f++) {
        size_t r = 0, w = 0;
        while (r < l.len) {
            Enemy v = l.data[r];
            v.x += v.vx; v.hp -= 1;
            calls++;
            if (v.hp <= 0) { list_push(&l, (Enemy){next++, 2 + (int32_t)(h_rand() % 100), 0.0f, 1.0f}); r++; continue; }
            l.data[w] = v; w++; r++;
        }
        l.len = w;
    }
    printf("callback calls: %llu (checksum %llu)\n", (unsigned long long)calls, (unsigned long long)checksum(&l, next));
    return 0;
}
