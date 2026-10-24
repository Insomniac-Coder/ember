/* VARIANT, the first version of the Ember model (not the answer): the name `e` is the place
   enemies[r], updated in memory, then copied down to enemies[w]. Same result as ember.c; 4.5x slower
   for the store-forwarding reason given in plain_naive.c. Kept to show what the fix was. */
#include "common.h"
int main(void) {
    h_start();
    List l; fill(&l);
    uint32_t next = N;
    for (int f = 0; f < FRAMES; f++) {
        size_t r = 0, w = 0;
        while (r < l.len) {
            Enemy *e = &l.data[r];
            e->x += e->vx;
            e->hp -= 1;
            int removed = 0;
            if (e->hp <= 0) {
                list_push(&l, (Enemy){next, 2 + (int32_t)(h_rand() % 100), 0.0f, 1.0f});
                next += 1;
                e = &l.data[r];
                removed = 1;
            }
            if (!removed) { if (w != r) l.data[w] = l.data[r]; w++; }
            r++;
        }
        l.len = w;
    }
    h_report("a4-ember-naive", checksum(&l, next));
    return 0;
}
