#$ test: run-pass
#$ rules: OPT-2, RNG-4
#$ profiles: debug, release, shipping
#$ stdout: 130
#$ stdout: 10
#$ stdout: 96
# `[OPT-2]` with `[RNG-4]` — an index that is not `i + c` but whose largest
# value the range facts know gets the same entry test and unchecked copy:
# `i % 4` is at most 3, `n - 1 - i` at most `n - 1`, and an inner counter
# over `0..2` at most 1, which the outer loop's test covers.

fn ring(xs: Array[int], n: int) -> int:
    total = 0
    for i in 0..n:
        total += xs[i % 4]
    return total

fn rev_sum(xs: Array[int], n: int) -> int:
    total = 0
    for i in 0..n:
        total += xs[n - 1 - i]
    return total

fn nested(sums: Array[int], cells: Array[int]) -> int:
    total = 0
    for r in 0..3:
        row = sums[r]
        for c in 0..2:
            row = row + cells[c]
        total = total + row
    return total

fn main():
    println(ring([10, 20, 30, 40], 6))
    println(rev_sum([1, 2, 3, 4], 4))
    println(nested([1, 2, 3], [10, 20]))
