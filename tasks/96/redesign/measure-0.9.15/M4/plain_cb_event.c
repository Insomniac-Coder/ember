/* plain_cb_event.c: workload cb_event, plain C. The callback workload run 3 cites (AN-78; run 2's
   artifacts/redesign2/threads/bench/ffi.c model 0, unchanged in substance): a C event library
   `fire(cb, ud, n)`, not inlined, calls the callback through a pointer it reads from a volatile (so
   the C compiler cannot see which function it is) 200,000,000 times; the callback adds a point to the
   game. The user data is the `Game *`. */
#include "m4.h"

#define CALLS 200000000

typedef void (*cb_t)(void *);
static NOINLINE void fire(cb_t cb, void *ud, int n) {           /* the C library */
    for (int i = 0; i < n; i++) cb(ud);
}

typedef struct { int64_t score; } Game;
static void on_event(void *ud) { ((Game *)ud)->score += 1; }

static cb_t volatile the_cb;
static void *volatile the_ud;

int main(void) {
    h_start();
    Game *g = (Game *)calloc(1, sizeof(Game));
    the_cb = on_event; the_ud = g;
    fire(the_cb, the_ud, CALLS);
    h_report("plain_cb_event", (uint64_t)g->score);
    return 0;
}
