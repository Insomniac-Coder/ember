/* salvage_w2v_freeonly.c (M2, workload "w2v"): a deferred-FREE region for a class that has a `drop`
   method of its own, the case where [WK-18] allows no deferred-drop region and the walk pays a count
   move per step (model_w2v_move_drop). Compare with model_w2v_move_drop (same class, same two header
   tests per step).

   What changes: the walk is a region in which an object whose last owner ends is ended and dropped at
   once, exactly as without the region (its `drop` runs at the same moment, so no visible action
   moves); only the free of its memory block, when its weak count reaches 0, waits for the region's
   end. Every link the walk makes after its first is then given no weak count.

   Safety: a weak count on the walk's link exists only so that the block it names is still there for
   the next header test. In the region no Node block is freed (the free path tests the per-thread
   region counter and sets the block aside), and the link lives only inside the region, so every header
   test reads a block that is still allocated; it finds the object ended (count 0) if the call ended
   it, and the walk stops, as the rule's walk does. The objects are one thread's (not `thread_shared`,
   not reachable from one: [WK-18]'s own condition for a region), so no other thread frees a block
   meanwhile. Cost moved off the walk: one test of the region counter on each block free of a class
   some walk keeps uncounted links to; memory held: the blocks freed during one walk, until it ends. */
#define FORM_FREEONLY
#define DROPCLASS
#include "model_w2v.c"
