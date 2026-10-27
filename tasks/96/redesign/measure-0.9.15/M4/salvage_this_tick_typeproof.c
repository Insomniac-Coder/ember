/* salvage_this_tick_typeproof.c: the model with `restrict` on `this` where a type proof allows it.

   Idea: verdict 18 forbids assuming `this` is its object's only name, because a link may lead back
   to it (a turret aimed at itself). [SIMD-3] already lets `this` take an alias fact "when the compiler
   proves that no other path the loop uses reaches the same object (the two have different types ...)".
   Widen that to the whole method: give `this` noalias when no pointer the method and everything it
   calls dereferences can lead to an object of `this`'s class (or of a base class or interface the
   class has). Here `tick` and `note` reach only `Player` objects (another class, its own block) and
   the int buffer `log` owns; neither is or contains a `Game`, so no second name of the game is used
   while `this` is: the promise `restrict` makes holds.

   Safety: the proof is the compiler's own place analysis (R5 works out every place a function
   reaches, through its calls, overrides and stored functions). The turret method dereferences a link
   whose type is its own class, so the proof fails there and it keeps today's emission. Nothing runs
   while the program runs. Identical code to the model otherwise. */
#define NOALIAS_THIS 1
#define NAME "salvage_this_tick_typeproof"
#include "model_this_tick.c"
