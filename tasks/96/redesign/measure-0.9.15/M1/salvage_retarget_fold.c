/* salvage_retarget_fold.c: salvage: the limit compare read from the increment's result (one fused add-and-branch), same limit and stop
   See counts.h for what VARIANT V_FOLD emits and the rule each line follows. */
#define VARIANT V_FOLD
#define NAME "salvage-retarget-fold"
#include "counts.h"
#include "retarget_body.h"
