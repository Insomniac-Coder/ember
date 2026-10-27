#$ test: run-pass
#$ rules: R1
#$ stdout: end one
#$ after
## R1: the nearest program that is allowed: `mem.drop(a)` ends `a` now, and its `drop` runs then, once
## (verdict 58: a value always ends with its clean-up).

import std.mem

class Res:
    name: String

    fn drop():
        println("end", this.name)

fn main():
    a = Res(name=String("one"))
    mem.drop(a)
    println("after")
