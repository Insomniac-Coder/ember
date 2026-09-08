#$ test: run-pass
#$ rules: MOD-1, MOD-2, MOD-3

# `[MOD-3]` — `from a.b import x, y as z` binds items; `import a.b.c` binds a
# namespace. `[MOD-1]` maps `modules.geom.shapes` to `modules/geom/shapes.em`.
from modules.geom.shapes import Point, area
from modules.util import double, helper as one
import modules.util

# A name this module also declares. The qualified symbols keep the two apart.
fn helper() -> i32:
    return 2

fn main():
    p = Point(3, 4)
    println(p.x + p.y)
    println(area(p.x, p.y))
    println(double(21))

    # The imported `helper` and this module's own `helper` are different
    # functions with the same written name.
    println(one())
    println(helper())

    # The namespace form reaches the same function.
    println(util.double(5))
#$ stdout: 7
#$ 12
#$ 42
#$ 1
#$ 2
#$ 10
