/* salvage_cb_once_nomark.c: the used-up mark proved unneeded.

   Idea: a second call of a `callback(once)` function can come only (a) while its one call runs, or
   (b) after it. (b) is stopped by the generation compare: the trampoline ends the registration when
   the body returns (the next generation), and the cold path names the once case (the slot's previous
   generation ended by its one call). (a) needs C code to run on this thread during the body (on
   another thread the creator compare stops it first, `threads(creator)`): the body must call into C.
   The compiler knows the body's calls (R5 works out every function a body may run); this body,
   `TOTAL += id`, calls nothing foreign. So for a `threads(creator)` or `threads(main)` once callback
   whose body makes no foreign call, the mark is never needed: no store, no test.

   Safety: the two ways a second call can happen are each stopped by a check that remains (the
   generation compare for (b); for (a), it cannot happen without a foreign call). A body that may call
   into C, or a `threads(any)` callback, keeps the mark (model or genmark form). */
#define ONCE_NOMARK 1
#define NAME "salvage_cb_once_nomark"
#include "model_cb_once.c"
