#$ test: run-pass
#$ rules: MOD-4
#$ stdout: 2
# `[MOD-4]` — import cycles within a package are allowed: `lib.a` imports `lib.b` and `lib.b`
# imports `lib.a`.

import lib.a

fn main():
    println(a.down(3))
