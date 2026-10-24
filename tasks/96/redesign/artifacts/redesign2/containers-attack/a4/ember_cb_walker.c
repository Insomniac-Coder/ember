/* A4 timing, unseen call in the body (plain_cb.c has the workload): the FINAL SAVED form. The walking
   state is in the field's access word ([EXC-19]); the loop publishes r and w in its frame (two stores
   before the call) and code that reaches the walked list through its field sees the logical list
   (walker_check.c checks the mapping): here the callback's len(self.enemies) is the logical length.
   Since no other code can see the gap, the gap needs no swap-fill: the plain one-pass filter, with e
   spilled before the call and reloaded field by field after it, and r, w read back after it. */
#include "../../containers/bench/a4/common.h"
#define WRITER 0x80000000u
#define WALKING 0x40000000u
static void panic(const char *m) { fprintf(stderr, "panic: %s\n", m); exit(101); }
typedef struct { size_t r, w; int cur_removed; } Walker;
static uint32_t enemies_word;                 /* World.enemies' access word */
static Walker *enemies_walker;                /* the walking loop's frame, while enemies_word says WALKING */
static volatile uint64_t sink;
static void on_step(List *l) {                /* len(self.enemies), through the field */
    uint32_t wd = enemies_word;
    if (__builtin_expect(wd & WRITER, 0)) panic("read access while written");
    size_t n = l->len;
    if (wd & WALKING) n -= (enemies_walker->r - enemies_walker->w) + (size_t)enemies_walker->cur_removed;
    sink += n;
}
static void (*volatile cb)(List *) = on_step;
int main(void) {
    h_start();
    List l; fill(&l);
    uint32_t next = N;
    for (int f = 0; f < FRAMES; f++) {
        if (__builtin_expect(enemies_word & (WRITER | WALKING), 0)) panic("walk while written or walked");
        enemies_word |= WALKING;
        Walker wk = {0, 0, 0};
        enemies_walker = &wk;
        size_t r = 0, w = 0;
        while (r < l.len) {
            Enemy e = l.data[r];
            e.x += e.vx; e.hp -= 1;
            l.data[r] = e;                                    /* spill e */
            wk.r = r; wk.w = w; wk.cur_removed = 0;           /* publish the walker */
            cb(&l);
            r = wk.r; w = wk.w;                               /* read back: other code may have removed */
            if (__builtin_expect(!(r < l.len && w <= r), 0)) panic("list changed under the loop");
            { volatile Enemy *vp = &l.data[r]; e.id = vp->id; e.hp = vp->hp; e.x = vp->x; e.vx = vp->vx; }  /* reload */
            int removed = wk.cur_removed;
            if (e.hp <= 0) { list_push(&l, (Enemy){next, 2 + (int32_t)(h_rand() % 100), 0.0f, 1.0f}); next += 1; removed = 1; }
            if (!removed) { l.data[w] = e; w++; }
            r++;
        }
        l.len = w;
        enemies_walker = NULL;
        enemies_word &= ~WALKING;
    }
    h_report("a4-cb-ember-walker", checksum(&l, next));
    return 0;
}
