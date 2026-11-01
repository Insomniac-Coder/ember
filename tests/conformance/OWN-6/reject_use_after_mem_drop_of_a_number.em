#$ test: compile-fail
#$ rules: OWN-6, OWN-9
# `[OWN-6]` — `mem.drop(x)` ends `x` now, for every type, numbers included.

from std.mem import drop

fn main():
    k = 7
    drop(k)
    println(k)  #$ error[E3040]
