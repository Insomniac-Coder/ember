/* salvage_this_tally_typeproof.c: the model with `restrict` on `this`, by the type proof of
   salvage_this_tick_typeproof.c: `tally` dereferences only the two int buffers its object owns
   (`samples`, `hist`); an owned buffer is never the object's own block, and no pointer the method
   uses can lead to a `Stats`, so `this` is the only name of the object while the method runs.
   Safety argument and the case where the proof fails: as in salvage_this_tick_typeproof.c. */
#define NOALIAS_THIS 1
#define NAME "salvage_this_tally_typeproof"
#include "model_this_tally.c"
