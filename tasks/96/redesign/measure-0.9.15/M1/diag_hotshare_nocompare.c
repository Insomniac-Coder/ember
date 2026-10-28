/* diag_hotshare_nocompare.c: diagnostic: model_hotshare.c without the limit compare, to isolate the compare's cost
   See counts.h for what VARIANT V_NOCMP emits and the rule each line follows. */
#define VARIANT V_NOCMP
#define NAME "diag-hotshare-nocompare"
#include "counts.h"
#include "hotshare_body.h"
