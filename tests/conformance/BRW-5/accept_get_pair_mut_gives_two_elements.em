#$ test: run-pass
#$ rules: BRW-5, STD-15, SPN-5
#$ stdout: [20, 10, 3]
#$ stdout: [3, 10, 20]
#$ stdout: none none none none
#$ stdout: [0, 1]
#$ stdout: 2
# `[BRW-5]`, `[STD-15]` (ODR-068, SP-031) — `get_pair_mut(i, j)` borrows the
# array once and gives mutable references to two different elements, in
# either order. An index out of range, or the same index twice, is `None`
# and changes nothing; so is any pair of an empty array. Zero-sized elements
# work as any other.

fn swap_through(mut xs: Array[int], i: int, j: int):
    match xs.get_pair_mut(i, j):
        Some((a, b)):
            t = a
            a = b
            b = t
        None:
            pass

fn describe(pair: Option[(ref mut int, ref mut int)]) -> str:
    return "some" if pair.is_some() else "none"

fn main():
    xs = [10, 20, 3]
    swap_through(xs, 0, 1)
    println(xs)
    match xs.get_pair_mut(2, 0):
        Some((last, first)):
            first = 3
            last = 20
        None:
            pass
    println(xs)
    empty: Array[int] = []
    println(describe(xs.get_pair_mut(1, 1)), describe(xs.get_pair_mut(0, 3)),
            describe(xs.get_pair_mut(-1, 0)), describe(empty.get_pair_mut(0, 1)))
    ys = [0, 0]
    match ys.get_pair_mut(0, 1):
        Some((_, b)):
            b = 1
        None:
            pass
    println(ys)
    units = [(), (), ()]
    count = 0
    if units.get_pair_mut(0, 2).is_some():
        count += 1
    if units.get_pair_mut(1, 0).is_some():
        count += 1
    if units.get_pair_mut(1, 1).is_some():
        count += 1
    println(count)
