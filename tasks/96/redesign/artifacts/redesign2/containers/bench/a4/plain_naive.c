/* Plain C, VARIANT (not the baseline): the in-place filter written the obvious way, updating the
   element where it is and then copying it down. Measured 4.5x slower than plain.c: the 16-byte copy
   (movups) loads an element that two 4-byte stores have just written, so the load cannot be forwarded
   from the store buffer and waits for the stores (checked in the assembly and with fieldcopy, see
   phase3b-containers.md). Rust's retain_mut has the same shape and the same time.
   Otherwise: an in-place filter with a read position r and a
   write position w over the live length. No checks. */
#include "common.h"
int main(void) {
    h_start();
    List l; fill(&l);
    uint32_t next = N;
    for (int f = 0; f < FRAMES; f++) {
        size_t r = 0, w = 0;
        while (r < l.len) {
            Enemy *e = &l.data[r];
            e->x += e->vx; e->hp -= 1;
            if (e->hp <= 0) {
                list_push(&l, (Enemy){next++, 2 + (int32_t)(h_rand() % 100), 0.0f, 1.0f});
                r++; continue;                       /* removed: not copied down */
            }
            if (w != r) l.data[w] = l.data[r];
            w++; r++;
        }
        l.len = w;
    }
    h_report("a4-plain-naive", checksum(&l, next));
    return 0;
}
