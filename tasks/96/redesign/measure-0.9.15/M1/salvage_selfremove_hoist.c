/* salvage_selfremove_hoist.c: model_selfremove.c without [M2]: the reads of the state after the call
   move above it, so no count is held (see selfremove_model.h and the report's safety argument).
   Checksum 279,575,944,000. */
#define NAME "salvage-selfremove-hoist"
#define HOIST
#include "selfremove_model.h"
