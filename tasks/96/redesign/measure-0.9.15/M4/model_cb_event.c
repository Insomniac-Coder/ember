/* model_cb_event.c: workload cb_event, what Ember emits: AN-78's program with v5's words.

       class Game:
           score: int = 0
           fn add_point():
               this.score += 1

       unsafe extern "C":
           fn unregister(cb: extern "C" fn(*mut void), ud: *mut void)
           @ffi(param(cb, callback(retained, user_data=ud), threads(creator)), destroys(unregister))
           safe fn register_cb(cb: extern "C" fn(*mut void), ud: *mut void)

       fn setup(game: shares mut Game) -> Retained[ForeignCallback]:
           return register_cb(fn() shares mut game: game.add_point())

   The C library then calls the callback 200,000,000 times (plain_cb_event.c's `fire`).

   Each call goes through the checked trampoline (cb_rt.h; [FFI-53], R15): attach ([FFI-22]); the
   slot inside the table; generation and type in one compare; the creator thread, in every profile
   ([FFI-53], [FFI-33b]). The body: `game.add_point()`, whose `+= 1` is checked ([TYP-8]). The
   `shares mut` capture is counted once, at registration (R9), not per call.
   -DNO_THREAD_CHECK: without the thread compare (today's release profile).
   salvage_cb_event_fold.c and salvage_cb_event_teb.c include this file with their mode. */
#include "cb_rt.h"

#define CALLS 200000000
#define TYPE_ADD_POINT 17u                                      /* the id the compiler gives this trampoline */

#ifndef NAME
#ifdef NO_THREAD_CHECK
#define NAME "model_cb_event_nothread"
#else
#define NAME "model_cb_event"
#endif
#endif

typedef void (*cb_t)(void *);
static NOINLINE void fire(cb_t cb, void *ud, int n) {           /* the C library, as plain C's */
    for (int i = 0; i < n; i++) cb(ud);
}

typedef struct { uint32_t count, type; int64_t score; } Game;  /* [OBJ-1] */
static FORCEINLINE void Game_add_point(Game *this_) { this_->score = ck_add(this_->score, 1); }   /* [TYP-8] */

static void tramp_add_point(void *ud) {
    Slot *s = cb_enter(ud, TYPE_ADD_POINT);
    Game_add_point((Game *)s->env[0]);                          /* the lambda's body, its state in the slot */
}

static cb_t volatile the_cb;
static void *volatile the_ud;

int main(void) {
    h_start();
    cb_init(64);
    Game *g = (Game *)calloc(1, sizeof(Game));
    g->count = 1; g->type = 3;
    g->count += 1;                                              /* R9: the callback is one more owner */
    uint64_t h = cb_register(TYPE_ADD_POINT, 0, g, 0);          /* register_cb(...) */
    the_cb = tramp_add_point; the_ud = (void *)(uintptr_t)h;
    fire(the_cb, the_ud, CALLS);
    cb_end(h & SLOT_MASK);                                      /* the token ends: unregistered */
    h_report(NAME, (uint64_t)g->score);
    return 0;
}
