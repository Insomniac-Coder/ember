/* The C side of W6's R15 tests of the checked trampoline (AN-78): a C library that keeps one
   callback, runs it on the calling thread or on a thread of its own, keeps a stale copy after it is
   unregistered (a library defect), says what number it was given as user data, and runs a
   borrowed callback during the call. */
#ifndef KEPT_EVENTS_H
#define KEPT_EVENTS_H
#include <stdint.h>

typedef void (*ke_fn)(void* data);

/* Keeps `cb` and `ud` until `ke_unregister` with the same two; also keeps a stale copy. */
void ke_register(ke_fn cb, void* ud);
void ke_unregister(ke_fn cb, void* ud);
/* Runs the kept callback, if any, on the calling thread. */
void ke_poll(void);
/* Runs the kept callback, if any, on a thread of its own, and waits for it. */
void ke_poll_elsewhere(void);
/* The defect: runs the stale copy, even after `ke_unregister`. */
void ke_fire_stale(void);
/* The top 16 bits of the user data C was last given. */
uint32_t ke_last_tag(void);
/* Runs `cb(ud)` `times` times during the call. */
void ke_each(ke_fn cb, void* ud, int32_t times);
#endif
