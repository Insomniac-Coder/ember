/* salvage_hotshare_wide.c: salvage: (owners-1) in the top 45 bits of one 64-bit header word, no compare (owners are bounded by memory)
   See counts.h for what VARIANT V_WIDE emits and the rule each line follows. */
#define VARIANT V_WIDE
#define NAME "salvage-hotshare-wide"
#include "counts.h"
#include "hotshare_body.h"
