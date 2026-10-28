/* model_cb_once.c: workload cb_once, what Ember emits.

       static mut TOTAL: int = 0

       unsafe extern "C":
           @ffi(param(cb, callback(once, user_data=ud), threads(creator)))
           safe fn lib_request(cb: extern "C" fn(*mut void), ud: *mut void)
           safe fn lib_pump()

       fn main():
           next_id = 0
           for b in 0..48828:
               for k in 0..1024:
                   id = next_id
                   lib_request(fn() copy of id: TOTAL += id)
                   next_id += 1
               lib_pump()

   Per request: registration ([FFI-21], [FFI-53]): a free slot, its generation, the trampoline's type,
   the creator thread, the state (`copy of id`) in the slot, the `once` flag. Per callback: the checked
   trampoline (cb_rt.h): attach ([FFI-22]); the slot inside the table; generation and type in one
   compare; the creator thread in every profile ([FFI-53], [FFI-33b]); then the used-up mark of a
   `callback(once)` slot ([FFI-53]): tested (a second call while this one runs stops the program) and
   set; the body (`TOTAL += id`, checked, [TYP-8]; `TOTAL` is a module variable the main thread reaches,
   R15); then, its one call over, the function value ends ([FFI-21]) and the slot's registration ends
   (the next generation, so a later call finds it stale and stops, with the once message).
   `lib_pump` is a foreign call that may run the callbacks C keeps (R12); nothing here walks what they
   change, so nothing is refused.

   Modes: -DNO_THREAD_CHECK (today's release: no thread compare); salvages: -DONCE_GENMARK (the mark
   folded into the compared word), -DONCE_NOMARK (the mark proved unneeded), through
   salvage_cb_once_genmark.c and salvage_cb_once_nomark.c. */
#include "cb_rt.h"
#include "cb_once_lib.h"

#define TYPE_DONE 31u

#ifndef NAME
#ifdef NO_THREAD_CHECK
#define NAME "model_cb_once_nothread"
#else
#define NAME "model_cb_once"
#endif
#endif

int64_t TOTAL;                                                  /* `static mut TOTAL` */

static void tramp_done(void *ud) {
    Slot *s = cb_enter(ud, TYPE_DONE);
#if defined(ONCE_GENMARK)
    s->word |= USED_BIT;                                        /* a second call fails the one compare */
#elif defined(ONCE_NOMARK)
    /* no mark: proved unneeded (salvage_cb_once_nomark.c) */
#else
    if (UNLIKELY(s->flags & F_USED)) cb_stop_once();           /* [FFI-53] the used-up mark: tested */
    s->flags |= F_USED;                                         /* and set */
#endif
    int64_t id = (int64_t)(intptr_t)s->env[0];                  /* `copy of id`, in the slot */
    TOTAL = ck_add(TOTAL, id);                                  /* [TYP-8] */
    cb_end((uint64_t)(s - g_table));                            /* [FFI-21] once: its one call is over */
}

static cb_t volatile the_cb = tramp_done;

int main(void) {
    h_start();
    cb_init(BATCH + 64);
    int64_t next_id = 0;
    for (int b = 0; b < BATCHES; b++) {
        cb_t cb = the_cb;
        for (int k = 0; k < BATCH; k++) {
            int64_t id = next_id;
            uint64_t h = cb_register(TYPE_DONE, F_ONCE, (void *)(intptr_t)id, 0);
            lib_request(cb, (void *)(uintptr_t)h);
            next_id = ck_add(next_id, 1);                       /* [TYP-8] */
        }
        lib_pump();
    }
    h_report(NAME, (uint64_t)TOTAL + (uint64_t)next_id);
    return 0;
}
