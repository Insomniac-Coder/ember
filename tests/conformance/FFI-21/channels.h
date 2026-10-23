/* The C side of FFI-21's test of a nullable retained callback undone by one of the registering
   call's arguments and its result. */
#ifndef CHANNELS_H
#define CHANNELS_H

typedef int (*tick_fn)(void* data, int value);

/* Keeps `hook` for `channel` until `unsubscribe_on(channel, id)` with the id this returns; a null
   hook keeps nothing and returns 0 (`removed()` then says -2 for null data too, -3 otherwise). */
int subscribe_on(int channel, tick_fn hook, void* data);
/* Removes the hook of `channel` when `id` is its id; `removed()` says `channel * 100 + id`. */
void unsubscribe_on(int channel, int id);
/* The kept hook of `channel` called with `value`, or -1. */
int tick(int channel, int value);
int removed(void);
#endif
