/* A4 timing, unseen call in the body: plain C written to be CORRECT FOR ANY CALLBACK the Ember program
   allows (one that may read or write the current element through the list, read the list's logical
   length, or remove elements through the published walker), with no safety checks: no access word,
   no re-check panics. What a C programmer must write when the callback is not known: publish r, w
   and the removed flag, store the element to its slot before the call, read r, w and the flag back
   and reload the element after it (at the widths of the stores). Compare with plain_cb_logical.c
   (C that knows the callback only reads the length) and ember_cb_walker.c (the first attack's saved
   Ember form: this plus the access word and the two re-check panics).
   Workload: ../../containers/bench/a4/common.h; callback as in plain_cb_logical.c. */
#include "../../containers/bench/a4/common.h"
typedef struct { size_t r, w; int cur_removed; } Walker;
static Walker *walker;
static volatile uint64_t sink;
static void on_step(List *l) { sink += l->len - (walker->r - walker->w) - (size_t)walker->cur_removed; }
static void (*volatile cb)(List *) = on_step;
int main(void) {
    h_start();
    List l; fill(&l);
    uint32_t next = N;
    for (int f = 0; f < FRAMES; f++) {
        Walker wk = {0, 0, 0};
        walker = &wk;
        size_t r = 0, w = 0;
        while (r < l.len) {
            Enemy e = l.data[r];
            e.x += e.vx; e.hp -= 1;
            l.data[r] = e;                                    /* the callback may read it */
            wk.r = r; wk.w = w; wk.cur_removed = 0;
            cb(&l);
            r = wk.r; w = wk.w;                               /* the callback may have removed */
            { volatile Enemy *vp = &l.data[r]; e.id = vp->id; e.hp = vp->hp; e.x = vp->x; e.vx = vp->vx; }  /* or written it */
            int removed = wk.cur_removed;
            if (e.hp <= 0) { list_push(&l, (Enemy){next, 2 + (int32_t)(h_rand() % 100), 0.0f, 1.0f}); next += 1; removed = 1; }
            if (!removed) { l.data[w] = e; w++; }
            r++;
        }
        l.len = w;
        walker = NULL;
    }
    h_report("a4-cb-plain-any", checksum(&l, next));
    return 0;
}
