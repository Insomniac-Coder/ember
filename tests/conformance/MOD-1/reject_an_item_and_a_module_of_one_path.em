#$ test: compile-fail
#$ rules: MOD-1, MNG-1
#$ error[E1032]: item `inner.cells` has the path of module `inner.cells`
#$ help: rename the item, or the module's file
# ODR-201 — a module and an item of one name in one scope is `E1032`, so every item has one path
# (`[MNG-1]`). This module's `inventory` and the module `inventory.em` beside it are both
# `inventory`, so `inventory.count` would name the struct's method and the module's function; module
# `inner`'s `cells` (in `inner/mod.em`, reported there) and the module `inner/cells.em` are both
# `inner.cells`. Before, the first pair was one C function defined twice.

import inventory
import inner
import inner.cells

struct inventory:  #$ error[E1032]: item `inventory` has the path of module `inventory`
    n: int

    fn count(self) -> int:
        return self.n

fn main():
    println(inventory(n = 1).count(), inner.cells(), cells.total())
