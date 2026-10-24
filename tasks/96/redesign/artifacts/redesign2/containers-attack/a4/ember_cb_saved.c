/* A4 timing, unseen call in the body (plain_cb.c has the workload): the SAVED answer. The walking state
   is in the field's access word ([EXC-19]), not in the list: the loop sets it at its start and clears it
   at its end (a held access: load, compare, store); the callback's read of len checks the word (a load
   and a compare, not counted). e is spilled to its slot before the call and reloaded after; a removed
   element is stored back to its slot (owning elements must never leave a stale copy); swap-fill; the
   re-checks of rule 10 stay. push is the plain one: nothing lives in the capacity word. */
#include "../../containers/bench/a4/common.h"
#define WRITER 0x80000000u
#define WALKER 0x00010000u
static void panic(const char *m) { fprintf(stderr, "panic: %s\n", m); exit(101); }
static volatile uint64_t sink;
static uint32_t enemies_word;                                 /* World.enemies' access word */
static void on_step(List *l) {
    if (__builtin_expect(enemies_word & WRITER, 0)) panic("read access while written");
    sink += l->len;
}
static void (*volatile cb)(List *) = on_step;
int main(void) {
    h_start();
    List l; fill(&l);
    uint32_t next = N;
    for (int f = 0; f < FRAMES; f++) {
        if (__builtin_expect(enemies_word & WRITER, 0)) panic("walk while written");
        enemies_word += WALKER;
        size_t r = 0, w = 0;
        while (r < l.len) {
            Enemy e = l.data[r];
            e.x += e.vx; e.hp -= 1;
            l.data[r] = e;                                    /* spill */
            cb(&l);
            if (__builtin_expect(!(r < l.len && w <= r), 0)) panic("list changed under the loop");
            e = l.data[r];                                    /* reload */
            int removed = 0;
            if (e.hp <= 0) { list_push(&l, (Enemy){next, 2 + (int32_t)(h_rand() % 100), 0.0f, 1.0f}); next += 1; removed = 1; }
            if (!removed) { if (w != r) l.data[r] = l.data[w]; l.data[w] = e; w++; }
            else l.data[r] = e;                               /* store back */
            r++;
        }
        l.len = w;
        enemies_word -= WALKER;
    }
    h_report("a4-cb-ember-saved", checksum(&l, next));
    return 0;
}
