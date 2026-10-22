/* The C side of the FFI-33b tests of a callback bound to its creator's thread. */
#ifndef TICKS_H
#define TICKS_H

typedef void (*tick_fn)(void* data);

/* Calls `tick(data)` on the calling thread. */
void tick_here(tick_fn tick, void* data);
/* Calls `tick(data)` on a thread of its own, and waits for it. */
void tick_elsewhere(tick_fn tick, void* data);
#endif
