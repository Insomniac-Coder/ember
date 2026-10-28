/* salvage_this_mix_typeproof.c: the model with `restrict` on `this`, by the type proof of
   salvage_this_tick_typeproof.c: `process` dereferences only the two float buffers its object owns;
   neither is the object's block, and no pointer it uses can lead to a `Mixer`. Safety argument and
   the case where the proof fails: as in salvage_this_tick_typeproof.c. */
#define NOALIAS_THIS 1
#define NAME "salvage_this_mix_typeproof"
#include "model_this_mix.c"
