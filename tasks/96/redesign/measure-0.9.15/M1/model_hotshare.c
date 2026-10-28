/* model_hotshare.c: the rule as written for a `shares mut` owner: 8-byte header, compare with 4,294,967,295 before each +1 ([RC-1], [RT-7], R9), plain -1 ([RT-8]), bounds check as today
   See counts.h for what VARIANT V_MODEL emits and the rule each line follows. */
#define VARIANT V_MODEL
#define NAME "model-hotshare"
#include "counts.h"
#include "hotshare_body.h"
