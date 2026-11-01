#$ test: run-pass
#$ rules: FN-2a
# `[FN-2a]` — the call says `ref mut` before an argument the callee may change, with the
# parameter's own words: `value` is the place `double` changes, and the write is visible afterwards.

fn double(ref mut n: i32):
    n = n + n

fn main():
    value: i32 = 2
    double(ref mut value)
    println(value)
#$ stdout: 4
