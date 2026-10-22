#$ test: compile-fail
#$ rules: MNG-1
#$ help: rename the module or the type
# `[MNG-1]` — a generic instance's symbol is its path's, then `G` and the first 16 hex digits of the
# BLAKE3 hash of its arguments; two instances whose symbols are one are `E9040`, naming both, never
# an internal compiler error or a C function defined twice. A collision of 64-bit hashes cannot be
# written down, so the case here is the other way two instances meet: the module `grid` beside this
# file and the type `grid` declared here both make the path `grid.at`, the module's generic function
# and the type's method, and over `i64` the two instances (`grid.at[i64]`, `grid[i64].at`) have one
# path and one list of arguments. The hash's own case is `ember_typeck`'s
# `two_instances_whose_hashes_meet_are_e9040`.

import grid

struct grid[T]:
    v: T

    fn at(self) -> int:  #$ error[E9040]: the generic instances `grid.at[i64]` and `grid[i64].at` hash to one symbol
        return 1

fn main():
    g = grid[int](3)
    println(g.at(), grid.at(4))
