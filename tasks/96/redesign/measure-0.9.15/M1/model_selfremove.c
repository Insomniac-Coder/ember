/* model_selfremove.c: W4 as 0.9.15 compiles it, with [M2]: the handler that removes itself holds one
   count on its own state across the call, compared with its maximum first (R12, AN-60, [RT-7]).
   See selfremove_model.h. Checksum 8 * 34,946,993,000 = 279,575,944,000. */
#define NAME "model-selfremove"
#include "selfremove_model.h"
