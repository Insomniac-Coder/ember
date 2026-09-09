#$ test: run-pass
#$ rules: MOD-3
## "`import a.b.c` binds `c` as a namespace; `from a.b import x, y as z` binds
## items; `import a.b.c as d` renames."

from std.math import min_i32, max_i32 as bigger
import std.math

fn main():
    println(min_i32(3, 7))
    println(bigger(3, 7))
    println(math.clamp_f32(2.5, 0.0, 1.0))
#$ stdout: 3
#$ 7
#$ 1
