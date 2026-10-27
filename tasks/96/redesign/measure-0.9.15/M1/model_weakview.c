/* model_weakview.c: the rule as written: a view of a `thread_shared` Player opened from a `weakref` link holds one atomic count for its life (R10 point 2, [RC-5] point 2, [WK-12], [RT-10])
   See weakview.h for what VARIANT W_TS emits and the rule each line follows. */
#define VARIANT W_TS
#define NAME "model-weakview"
#include "weakview.h"
