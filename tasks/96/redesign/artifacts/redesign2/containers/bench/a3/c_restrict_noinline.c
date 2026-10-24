/* VARIANT: c_restrict.c with collide kept out of line. */
#define COLLIDE_QUAL restrict
#define NOINLINE
#include "common.h"
int main(void) { return run("a3-restrict-noinline"); }
