/* rtalloc.c: stands for Ember's runtime allocator (ember_alloc in runtime/ember_rt/src/ember_rt.c,
   declared in ember_rt.h with no malloc attribute and compiled in its own translation unit), so the
   C compiler cannot assume a block it returns is fresh. Linked into the Ember models of boxloop.c. */
#include <stdlib.h>
void *rt_alloc(size_t n) { return malloc(n); }
