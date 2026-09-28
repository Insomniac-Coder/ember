#$ test: run-fail
#$ rules: STD-19, STD-14
#$ profiles: debug, release, shipping
#$ stdout: before
#$ panics: nth(-1): an index cannot be negative
# `[STD-19]` (ODR-089) — a bad count is the caller's bug and panics when
# `nth` is called, before any item is read: nothing after it prints.

fn main():
    xs: Array[int] = [1, 2, 3]
    println("before")
    it = xs.iter().nth(-1)
    println("after", it)
