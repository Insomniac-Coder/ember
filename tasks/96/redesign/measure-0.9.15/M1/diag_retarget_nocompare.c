/* diag_retarget_nocompare.c: diagnostic: model_retarget.c without the limit compare, to isolate the compare's cost
   See counts.h for what VARIANT V_NOCMP emits and the rule each line follows. */
#define VARIANT V_NOCMP
#define NAME "diag-retarget-nocompare"
#include "counts.h"
#include "retarget_body.h"
