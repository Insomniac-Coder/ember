/* A4 timing, unseen call in the body, plain C doing the SAME visible work as the saved answer
   (ember_cb_walker.c): the callback is told the loop's r and w so that len(self.enemies) is the
   logical length (what Python's callback sees), as a C programmer would do it: two stores before the
   call, the callback subtracts. No safety checks, no spill of the element. */
#include "../../containers/bench/a4/common.h"
typedef struct { size_t r, w; } Walk;
static Walk walk;
static volatile uint64_t sink;
static void on_step(List *l) { sink += l->len - (walk.r - walk.w); }
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
            walk.r = r; walk.w = w;
            cb(&l);
            if (v.hp <= 0) { list_push(&l, (Enemy){next++, 2 + (int32_t)(h_rand() % 100), 0.0f, 1.0f}); r++; continue; }
            l.data[w] = v; w++; r++;
        }
        l.len = w;
    }
    h_report("a4-cb-plain-logical", checksum(&l, next));
    return 0;
}
