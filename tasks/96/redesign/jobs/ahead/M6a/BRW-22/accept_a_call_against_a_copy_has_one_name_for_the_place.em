# Clause BRW-22.2: "a call compiled against a copy with one name (`[BRW-11]`, exception 1) has one
# name for that place"
#$ test: run-pass
#$ rules: BRW-22, BRW-11, MONO-10
#$ profiles: debug, development, release
#$ stdout: 2
#$ stdout: 4
#$ stdout: 11
# Run in every profile, the optimised one included: with two names for one place and noalias
# between them, the second `+= 1` could be computed from a stale first. The copy has one name, so
# `add_both` prints 2 and leaves 2, and `tally(ref mut TOTAL)` sees its own write through `n`. The
# noalias of every `ref mut` parameter (BRW-22.1) is M6c's: not here.

static mut TOTAL: int = 0

fn add_both(ref mut a: int, ref mut b: int):
    a += 1
    b += 1
    println(a)

fn tally(ref mut n: int):
    n += 1
    TOTAL += 10
    println(n)

fn main():
    x = 0
    add_both(ref mut x, ref mut x)
    add_both(ref mut x, ref mut x)
    TOTAL = 0
    tally(ref mut TOTAL)
