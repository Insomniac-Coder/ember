/* The C side of the FFI-21 tests of callbacks C keeps: one called once later, and listeners
   registered until they are removed. */
#ifndef EVENTS_H
#define EVENTS_H

typedef void (*done_fn)(void* data, int result);
typedef int (*listener_fn)(int event, void* data);

/* Keeps `done` and `data`; `run_pending` calls each kept one once, with `value * 2`. */
void run_later(int value, done_fn done, void* data);
int run_pending(void);

/* Registers `listener` for events of `kind` until `unlisten` with the same arguments. */
void listen(int kind, listener_fn listener, void* data);
void unlisten(int kind, listener_fn listener, void* data);
/* Calls each listener of `kind` with `event`; the sum of what they return. */
int emit(int kind, int event);
int listening(void);
#endif
