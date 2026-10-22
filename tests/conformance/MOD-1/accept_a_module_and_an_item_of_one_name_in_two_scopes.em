#$ test: run-pass
#$ rules: MOD-1, MNG-1
#$ stdout: 30 7 3 4
#$ assert-c: contains("struct em_4grid {")
#$ assert-c: contains("struct em_5paths6shapes4grid {")
# ODR-201 — a module and an item of one name in one scope is `E1032`; in two scopes they are two
# paths, and every item keeps one (`[MNG-1]`). This module's `grid` is `grid`, the module beside it
# is `paths.grid` (imported as `cells`), and module `paths.shapes`'s `grid` is `paths.shapes.grid`:
# one name, three scopes, three symbols. This module's `path` and the module `paths.grid` have
# different names.

import paths.grid as cells
import paths.shapes

struct grid:
    n: int

    fn at(self, i: int) -> int:
        return self.n + i

fn path() -> int:
    return 4

fn main():
    println(cells.at(3), shapes.grid(n = 3).at(4), grid(n = 1).at(2), path())
