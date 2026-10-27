#$ test: run-pass
#$ rules: R1
#$ stdout: 5
## R1: the nearest program that is allowed: the name is used before `mem.drop` ends it.

import std.mem

fn main():
    a = 5
    println(a)
    mem.drop(a)
