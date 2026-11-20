/* The C side of the FFI-53 tests: a library that keeps one callback and its user data, whose
   `unkeep` is deliberately wrong (it forgets nothing), so that `fire` can call a callback whose
   token has ended and a `once` callback a second time; `fire_elsewhere` calls the kept callback on
   another thread. */
#ifndef KEPT_H
#define KEPT_H
#include <stdint.h>

typedef int (*kept_fn)(void* data, int value);

/* Keeps `cb` and `data`, replacing what was kept. */
void keep(kept_fn cb, void* data);
/* The same, for a callback the caller says C runs once. */
void keep_once(kept_fn cb, void* data);
/* The same, for a callback the caller says C runs only on the thread that kept it. */
void keep_here(kept_fn cb, void* data);
/* Should forget `cb`; this test's C keeps it (the bug the trampoline catches). */
void unkeep(kept_fn cb, void* data);
/* Calls the kept callback with `value`; -1 when none is kept. */
int fire(int value);
/* Calls the kept callback with `value` on a thread of its own, waits for it, and returns its
   result. */
int fire_elsewhere(int value);
/* 1 when the user data kept last has bits 48, 55 and 63 set (on a 64-bit target), else 0; 1 on a
   32-bit target. */
int tagged(void);
#endif
