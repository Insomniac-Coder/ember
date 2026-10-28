/* salvage_w2v_hoist.c (M2, workload "w2v"): the deferred-drop region of model_w2v_region, entered once
   per pass of walks instead of once per walk, and the walk opening its leaf first
   (`if leaf exists as n: up = weakref n`), so even the first link is given a live node inside the
   region and holds no count. Per walk: nothing but the leaf's open (its header test); per step: the
   header test. Compare with model_w2v_region.

   Safety: R11's own rule for the region, applied twice. (1) A link that lives only inside the region
   and is given only objects that live at that moment holds no weak count: `n` is a view opened that
   instant, so `up = weakref n` qualifies, as `up = weakref p` does. Opening the leaf first runs the
   same test the first `while up exists as n` would, so the walk visits the same nodes. (2) A region
   around the pass keeps every object ended during the pass set aside (marked ended, its block kept)
   until the pass ends, so every uncounted link of every walk in it names a block still there. The
   class has no drop of its own and no other thread can end its objects, the region's conditions.
   Cost moved: the memory of every node ended during a pass is held until the pass ends, not the walk
   (here none: `prune` is never stored in the timed runs). */
#define FORM_HOIST
#include "model_w2v.c"
