# Clause MONO-10.1: "A call that the compiler proves, when compiling, names one whole variable twice
# — two `ref mut` arguments, or a `ref mut` argument and a module variable the callee uses by its
# own name (`add_both(ref mut x, ref mut x)`, `rename(ref mut LOG)`) — is compiled against its own
# copy of the callee in which the two names are one place, so the copy computes what its lines say
# for one variable"
#$ test: run-pass
#$ rules: MONO-10, BRW-11, BRW-22
#$ stdout: 2
#$ stdout: 2
#$ stdout: 11
# Both kinds of pair, with a module variable as the one whole variable: `add_both(ref mut TOTAL, ref
# mut TOTAL)` adds 1 twice to `TOTAL` and prints 2; `tally(ref mut TOTAL)` adds 1 through `n`, then
# 10 to `TOTAL` by its own name, and `n` is `TOTAL`, so from 0 it prints 11.

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
    add_both(ref mut TOTAL, ref mut TOTAL)
    println(TOTAL)
    TOTAL = 0
    tally(ref mut TOTAL)
