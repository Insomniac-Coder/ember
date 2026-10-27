/* diag_hotshare_ts_nocompare.c: diagnostic: model_hotshare_ts.c without the limit test, a C++ intrusive atomic
   count ([BEN-6]'s baseline for `thread_shared`), to isolate the test's cost.
   See counts.h for what VARIANT V_TS_NOCMP emits. */
#define VARIANT V_TS_NOCMP
#define NAME "diag-hotshare-ts-nocompare"
#include "counts.h"
#include "hotshare_body.h"
