/* MECHANISM CHECK (not a baseline): plain_naive.c with the copy written field by field, so no
   16-byte load reads an element just written by 4-byte stores. If the naive version is slow because
   of store forwarding, this one is fast. Measured: it is (about 130 ms against 450 ms). */
/* Plain C, the best one-pass order-keeping way: an in-place filter with a read position r and a
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
            if (w != r) { Enemy *d = &l.data[w], *s = &l.data[r]; d->id = s->id; d->hp = s->hp; d->x = s->x; d->vx = s->vx; }
            w++; r++;
        }
        l.len = w;
    }
    h_report("a4-plain-fieldcopy", checksum(&l, next));
    return 0;
}
