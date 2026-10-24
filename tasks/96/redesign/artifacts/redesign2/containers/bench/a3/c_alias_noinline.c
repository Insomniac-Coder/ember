/* VARIANT: c_alias.c with collide kept out of line, so the function body is compiled without
   knowing the two pointers (where `restrict` could matter most). */
#define COLLIDE_QUAL
#define NOINLINE
#include "common.h"
int main(void) { return run("a3-alias-noinline"); }
