/* The other answer for A3 (and Rust's get_disjoint_mut): check i != j, then two non-aliasing
   pointers (`restrict`). */
#define COLLIDE_QUAL restrict
#define CHECK_DISTINCT
#include "common.h"
int main(void) { return run("a3-check-restrict"); }
