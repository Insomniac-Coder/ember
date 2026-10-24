/* The answer for A3 (plain-data elements): the two names may be one body, so the pointers carry no
   `restrict`; no check. This is what C writes by default. */
#define COLLIDE_QUAL
#include "common.h"
int main(void) { return run("a3-alias"); }
