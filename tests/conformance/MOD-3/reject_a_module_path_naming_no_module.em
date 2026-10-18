#$ test: compile-fail
#$ rules: MOD-3, PHIL-12
#$ help: the nearest modules: `std.math`
#$ help: the nearest modules: `support.geometry`
# `[MOD-3]` — a module path that names no module is `E1060` at the import, a `std` path as much
# as any other: it is never accepted, and never left to the first use (F-004). Its help lists the
# nearest existing module paths.

import std.maths    #$ error[E1060]: no module named 'std.maths'
from support.geometri import Point    #$ error[E1060]: no module named 'support.geometri'

fn main():
    println(1)
