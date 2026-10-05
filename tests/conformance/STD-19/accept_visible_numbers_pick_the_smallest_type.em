#$ test: run-pass
#$ rules: STD-19, STD-26, CTL-3b
#$ stdout: 10 10 10 90
#$ stdout: 20 190 [(-3, 0), (-2, 1)]
#$ stdout: 10 18446744073709551615 [(0, 5)]
#$ stdout: -3
#$ profiles: debug, release
# G8-4 (the owner's design, 0.9.10) — the visible-numbers rule: where a range's numbers are written
# at the place it is numbered or measured, `enumerate` numbers and `len` and `count` measure in the
# smallest type that holds every value the loop can give, from the bounds written as numbers, the
# bounds' types, what `take`, `skip`, `step_by`, `zip` and `chain` allow, and `enumerate`'s start. `(0..n)` with `n: int` has at most `int.MAX` items, so
# its count and numbers are `int`s and `total += i` adds two `int`s; with `n: u64` they are `u64`s,
# and from a negative start `i128`s. A range kept in a variable shows no numbers: the table's type.
# `(lo..hi).step_by(2)` with `lo, hi: int` gives at most 2^63 items, so it numbers in `int`.

fn main():
    n = 10
    a: int = (0 .. 10).iter().len()
    b: int = (0 .. n).iter().len()
    c: int = len(0 .. n)
    total = 0
    for i, x in (0 .. n).iter().enumerate():
        total += i + x
    println(a, b, c, total)
    m: u64 = 20
    d: u64 = (0 as u64 .. m).iter().count()
    sum: u64 = 0
    for i, _x in (0 as u64 .. m).iter().enumerate():
        sum += i
    first: Array[(i128, u64)] = []
    for i, x in (0 as u64 .. m).iter().enumerate(-3):
        first.push((i, x))
        if first.len() == 2:
            break
    println(d, sum, first)
    kept = 0 .. n
    e: u64 = kept.iter().len()
    wide = i64.MIN .. i64.MAX
    f: u64 = wide.iter().len()
    g: Array[(int, int)] = (5 .. 6).iter().enumerate().to_array()
    println(e, f, g)
    lo = -5
    hi = 5
    stepped = 0
    for i, v in (lo .. hi).step_by(2).enumerate():
        stepped += i ^ v
    println(stepped)
