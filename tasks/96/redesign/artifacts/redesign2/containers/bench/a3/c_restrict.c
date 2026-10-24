/* What Rust's two `&mut` give LLVM (noalias), without Rust's distinctness check: `restrict`. */
#define COLLIDE_QUAL restrict
#include "common.h"
int main(void) { return run("a3-restrict"); }
