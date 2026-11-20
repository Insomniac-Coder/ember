# Clause BRW-21.8: "unless it is that same whole variable, proved when compiling (`[BRW-11]`,
# exception 1)"
#$ test: run-pass
#$ rules: BRW-21, BRW-11, MONO-10, BRW-22
#$ stdout: 2
#$ stdout: 6 3
# `bump(ref mut COUNT)` gives `n` the module variable `bump` also changes by its own name: one whole
# variable, so the call is compiled against a copy in which the two names are one, and both `+= 1`
# land on `COUNT`. `bump(ref mut x)` uses the ordinary body.

static mut COUNT: int = 0

fn bump(ref mut n: int):
    COUNT += 1
    n += 1

fn main():
    bump(ref mut COUNT)
    println(COUNT)
    x = 5
    bump(ref mut x)
    println(x, COUNT)
