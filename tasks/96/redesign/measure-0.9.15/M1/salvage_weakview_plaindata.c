/* salvage_weakview_plaindata.c: salvage: a `thread_shared` class whose end frees and runs nothing needs no view count; the opening is an atomic load and a test
   See weakview.h for what VARIANT W_PLAINDATA emits and the rule each line follows. */
#define VARIANT W_PLAINDATA
#define NAME "salvage-weakview-plaindata"
#include "weakview.h"
