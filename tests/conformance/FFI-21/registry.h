/* The C side of the FFI-21 tests of ODR-245, ODR-246 and ODR-253: a callback with two parameters
   of its user data's type, callbacks C may be given none of, and a registration undone by the id
   it returned. */
#ifndef REGISTRY_H
#define REGISTRY_H
#include <stdint.h>

/* Calls `visit(item, context, index)` for each of `count` items (`item` is `index * 10`); the sum
   of what it returns. Two parameters have the user data's type. */
typedef int (*pair_fn)(intptr_t item, intptr_t context, int index);
int each_pair(int count, pair_fn visit, intptr_t context);

/* The user data first and second: which is which only `@N` says. */
typedef int (*swap_fn)(void* left, void* right);
int both(swap_fn pick, void* right);

/* Calls `hook(data, value)` when `hook` is not null, else returns -1; `given()` says what the last
   call was given: 1 for a hook and data, 0 for neither, -2 for one without the other. */
typedef int (*hook_fn)(void* data, int value);
int maybe_call(hook_fn hook, void* data, int value);
int given(void);

/* Keeps `hook` until `unsubscribe` with the id this returns; a null hook keeps nothing and returns
   0. `fire` calls each kept one with `value`: the sum. */
int subscribe(hook_fn hook, void* data);
void unsubscribe(int id);
int fire(int value);
int subscribers(void);

/* Keeps `hook` to call once, at `run_deferred`; a null one is not kept. */
void defer(hook_fn hook, void* data);
int run_deferred(int value);
#endif
