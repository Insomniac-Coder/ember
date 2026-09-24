#$ test: run-pass
#$ rules: MOD-3
#$ stdout:
#$ 9
#$ 2 1
# `[MOD-3]` — a standard module may be named without `std.`, as in Python:
# `import math` is `import std.math`, and `from mem import swap` is
# `from std.mem import swap`.

import math
from mem import swap

fn main():
    println(math.max_i32(3, 9))
    a = 1
    b = 2
    swap(a, b)
    println(a, b)
