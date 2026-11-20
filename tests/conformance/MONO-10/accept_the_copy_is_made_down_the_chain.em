# Clause MONO-10.3: "The copy is made down the chain: every function the copy calls with the merged
# place under a `ref mut` parameter, and which reaches the other name while that parameter is used
# (its summary, `[BCK-8]`), is called through a copy of its own with the same pair made one, an
# instance like the first"
#$ test: run-pass
#$ rules: MONO-10, BCK-8
#$ stdout: 11
#$ stdout: 11
# `outer` reaches `TOTAL` only through `inner`'s summary; `outer(ref mut TOTAL)` is compiled against
# a copy of `outer`, whose call `inner(ref mut n)` gives `inner` the merged place, so `inner` is
# called through a copy of its own too: `m` and `TOTAL` are one, and it prints 11.

static mut TOTAL: int = 0

fn inner(ref mut m: int):
    m += 1
    TOTAL += 10
    println(m)

fn outer(ref mut n: int):
    inner(ref mut n)

fn main():
    outer(ref mut TOTAL)
    println(TOTAL)
