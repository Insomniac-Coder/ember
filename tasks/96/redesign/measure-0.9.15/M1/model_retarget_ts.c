/* model_retarget_ts.c: the rule as written for a `thread_shared` owner: one atomic add, limit tested on its result at 2,147,483,647 ([RC-4], [RT-7], [RT-10]); atomic -1 ([RT-8])
   See counts.h for what VARIANT V_TS emits and the rule each line follows. */
#define VARIANT V_TS
#define NAME "model-retarget-ts"
#include "counts.h"
#include "retarget_body.h"
