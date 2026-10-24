/* What the answer compiles this Ember loop into (A4 in tasks/96/redesign/phase3b-containers.md):

       for e in enemies:
           e.x += e.vx
           e.hp -= 1
           if e.hp <= 0:
               enemies.push(Enemy(id=next, hp=2 + rand() % 100, x=0.0, vx=1.0))
               next += 1
               enemies.remove(e)

   The loop changes its own list, so it is compiled as an in-place filter over the live length
   (Python's: an enemy appended during the loop is visited): read position r, write position w.
   Nothing but `e` touches enemies[r] during the iteration (the body is visible to the compiler and
   `push` only appends), so `e` is kept in registers: loaded once from enemies[r], stored once at
   enemies[w] when the iteration ends, or not stored if `enemies.remove(e)` ran. Every index is in
   range by construction (r < len is the loop test, w <= r), so no bounds check is left. Integer
   overflow checks on `hp -= 1` and `next += 1` are left out, as in every phase 3 model; plain C has
   none either.

   The first version of this model updated enemies[r] in place and then copied it down
   (ember_naive.c): 4.5x slower, for the store-forwarding reason given in plain_naive.c. Keeping `e` in
   registers is the fix, and it is what the compiler emits. */
#include "common.h"
int main(void) {
    h_start();
    List l; fill(&l);
    uint32_t next = N;
    for (int f = 0; f < FRAMES; f++) {
        size_t r = 0, w = 0;
        while (r < l.len) {                          /* live length */
            Enemy e = l.data[r];                     /* the name `e`, held in registers */
            e.x += e.vx;
            e.hp -= 1;
            int removed = 0;
            if (e.hp <= 0) {
                list_push(&l, (Enemy){next, 2 + (int32_t)(h_rand() % 100), 0.0f, 1.0f});
                next += 1;
                removed = 1;                         /* enemies.remove(e) */
            }
            if (!removed) { l.data[w] = e; w++; }    /* the iteration's end: e goes to its place */
            r++;
        }
        l.len = w;                                   /* the loop's end: the list closes up */
    }
    h_report("a4-ember", checksum(&l, next));
    return 0;
}
