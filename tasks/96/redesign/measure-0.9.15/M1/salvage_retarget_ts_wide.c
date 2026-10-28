/* salvage_retarget_ts_wide.c: salvage of the thread_shared count: the 45-bit word raised by one locked add, no limit test
   See counts.h for what VARIANT V_TS_WIDE emits and the rule each line follows. */
#define VARIANT V_TS_WIDE
#define NAME "salvage-retarget-ts-wide"
#include "counts.h"
#include "retarget_body.h"
