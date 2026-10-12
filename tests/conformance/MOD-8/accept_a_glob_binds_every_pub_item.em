#$ test: run-pass
#$ rules: MOD-8, MOD-3
#$ stdout: 9 shapes
## `[MOD-8]` — `from m import *` binds every `pub` item of `m`, from any
## module: it was `E1040`, a code the specification retired (D-582).

from support.shapes import *

fn main():
    println(area(Square(3)), describe())
