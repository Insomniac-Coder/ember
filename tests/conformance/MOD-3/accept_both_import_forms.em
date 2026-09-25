#$ test: run-pass
#$ rules: MOD-3
## "`import a.b.c` binds `c` as a namespace; `from a.b import x, y as z` binds
## items; `import a.b.c as d` renames."

from std.math import sqrt, cbrt as cube_root
import std.math

fn main():
    println(sqrt(9.0))
    println(cube_root(27.0))
    println(math.lerp(0.0, 1.0, 1.0))
#$ stdout: 3.0
#$ 3.0
#$ 1.0
