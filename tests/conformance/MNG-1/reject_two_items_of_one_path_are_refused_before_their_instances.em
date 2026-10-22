#$ test: compile-fail
#$ rules: MNG-1, MOD-1
#$ help: rename the item, or the module's file
# `[MNG-1]` — two distinct items never share a symbol, because no two share a path (ODR-201): the
# module `grid` beside this file and the type `grid` declared here would both make the path
# `grid.at`, the module's generic function and the type's method, and over `i64` the two instances
# (`grid.at[i64]`, `grid[i64].at`) one path and one list of arguments. The type is `E1032`, and the
# two instances are its cascade, not an `E9040` too (`[DIA-14]`). Before ODR-201 the program was
# `E9040` alone. The instance hash's own collision, which no program can write down, is
# `ember_typeck`'s `two_instances_whose_hashes_meet_are_e9040`.

import grid

struct grid[T]:  #$ error[E1032]: item `grid` has the path of module `grid`
    v: T

    fn at(self) -> int:
        return 1

fn main():
    g = grid[int](3)
    println(g.at(), grid.at(4))
