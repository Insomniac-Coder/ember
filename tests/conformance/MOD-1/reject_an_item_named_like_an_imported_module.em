#$ test: compile-fail
#$ rules: MOD-1
#$ help: rename the item, or import the module under another name with `as`
# ODR-201 — a module and an item of one name in one scope is `E1032`: `import paths.grid` binds
# `grid` to the module in this module's scope, where the function `grid` is declared, and `grid.at`
# would mean either. Before, the module silently won in `grid.at(3)`.

import paths.grid

fn grid() -> int:  #$ error[E1032]: item `grid` has the name of module `paths.grid`, which this module imports
    return 1

fn main():
    println(grid.at(3), grid())
