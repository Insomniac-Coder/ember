#$ test: run-pass
#$ rules: STD-19, STD-26, CTL-3b
#$ stdout: 10 10 10 90 10
#$ stdout: 20 190 [(-3, 0), (-2, 1)]
#$ stdout: 18446744073709551615 [(0, 5)]
#$ stdout: -3
#$ profiles: debug, release
# G8-4 (the owner's design B, 0.9.10), widened by the owner's rulings of 2026-10-06: the
# known-numbers rule. `enumerate` numbers, and `len` and `count` measure, in the smallest type
# that holds every value they can give, from the numbers the compiler knows: numbers written as
# numbers, names set once and never changed (`n = 10`, `kept = 0 .. n`, `lo` and `hi`), a loop's
# counter, arithmetic on them, and lengths. `(0..n)` has ten items, so its count and numbers are
# `int`s and `total += i` adds two `int`s. Where the compiler knows nothing of a number (`m`, a
# parameter), it may be any value of its type: `(0 as u64 .. m)` counts and numbers in `u64`,
# and from a negative start in `i128`. A whole `int` range has 2^64 - 1 items: a `u64` count.

fn wide(m: u64) -> String:
    d: u64 = (0 as u64 .. m).iter().count()
    sum: u64 = 0
    for i, _x in (0 as u64 .. m).iter().enumerate():
        sum += i
    first: Array[(i128, u64)] = []
    for i, x in (0 as u64 .. m).iter().enumerate(-3):
        first.push((i, x))
        if first.len() == 2:
            break
    return f"{d} {sum} {first}"

fn main():
    n = 10
    a: int = (0 .. 10).iter().len()
    b: int = (0 .. n).iter().len()
    c: int = len(0 .. n)
    total = 0
    for i, x in (0 .. n).iter().enumerate():
        total += i + x
    kept = 0 .. n
    e: int = kept.iter().len()
    println(a, b, c, total, e)
    println(wide(20))
    whole = i64.MIN .. i64.MAX
    f: u64 = whole.iter().len()
    g: Array[(int, int)] = (5 .. 6).iter().enumerate().to_array()
    println(f, g)
    lo = -5
    hi = 5
    stepped = 0
    for i, v in (lo .. hi).step_by(2).enumerate():
        stepped += i ^ v
    println(stepped)
