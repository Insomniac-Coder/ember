/* The alternative of last night's D2/D3 designs (and phase 1's A4(a)): removals are only marked
   during the loop and applied in a second pass at the loop's end (tombstones, then compaction).
   Spawns are still appended and visited (live length). Same work, same result, two passes.
   (The pin counters D3 adds are left out: this measures the two-pass shape alone.) */
#include "common.h"
int main(void) {
    h_start();
    List l; fill(&l);
    uint32_t next = N;
    for (int f = 0; f < FRAMES; f++) {
        size_t first_dead = (size_t)-1;
        for (size_t r = 0; r < l.len; r++) {
            Enemy *e = &l.data[r];
            e->x += e->vx; e->hp -= 1;
            if (e->hp <= 0) {                        /* the tombstone is hp <= 0 itself here */
                list_push(&l, (Enemy){next++, 2 + (int32_t)(h_rand() % 100), 0.0f, 1.0f});
                if (first_dead == (size_t)-1) first_dead = r;
            }
        }
        if (first_dead != (size_t)-1) {              /* settle: close the gaps */
            size_t w = first_dead;
            for (size_t r = first_dead; r < l.len; r++) if (l.data[r].hp > 0) l.data[w++] = l.data[r];
            l.len = w;
        }
    }
    h_report("a4-deferred", checksum(&l, next));
    return 0;
}
