#$ test: run-pass
#$ rules: MOD-3
## "`import a.b.c` binds `c` as a namespace; `from a.b import x, y as z` binds
## items; `import a.b.c as d` renames."

from std.math import sqrt, lerp as between
import std.math

fn main():
    println(sqrt(9.0))
    println(between(0.0, 6.0, 0.5))
    println(math.lerp(0.0, 1.0, 1.0))
#$ stdout: 3.0
#$ 3.0
#$ 1.0
