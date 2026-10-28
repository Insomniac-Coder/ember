/* salvage_cb_once_genmark.c: the used-up mark of a `callback(once)` slot kept inside the word the
   trampoline already compares.

   Idea: the model tests a separate flag and sets it (a load, a branch, a store). Instead, on entry
   the trampoline sets one bit in the slot's {generation, type} word, the word it has just compared:
   one store, no test. A second call while the first runs (C calling the callback again from inside
   it) compares `word` with {generation, type} and finds the bit: it fails the one compare and takes the
   cold path, which sees the bit and stops with the once message. After the call the registration ends
   (the next generation), so a later call fails the same compare.

   Safety: every call that the model's flag test stops, this compare stops (the bit is set before the
   body runs, exactly when the model's flag is); nothing else changes. */
#define ONCE_GENMARK 1
#define NAME "salvage_cb_once_genmark"
#include "model_cb_once.c"
