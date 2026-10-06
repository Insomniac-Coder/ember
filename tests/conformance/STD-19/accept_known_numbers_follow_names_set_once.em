#$ test: run-pass
#$ rules: STD-19, STD-26, CTL-3b, HEAP-8
#$ profiles: debug, release
#$ stdout:
#$ 999999 1000005
#$ 3999994000002 3999998000000 15999980000008
#$ 9223372036854775805 10 9223372036854775806 20 9223372036854775807 30
#$ 3 18
# `[STD-19]` (the owner's rulings of 2026-10-06) — the known-numbers rule follows a name to what it
# holds when it is set once and never changed: `n` is 2,000,000, and `round`, a loop's counter, is
# 0 to 2 (forwards, backwards, or stepped), so `(round..n)` has at most 2,000,000 items and its
# numbers are `int`s, with `step_by` or without. Arithmetic on known numbers is known (`n // 2 - 5`
# is 999,995), and so is a list's length: `ys` is set once from three items, so numbered from
# `int.MAX - 2` its numbers reach `int.MAX` and no further, and stay `int`s; `len` of a list is
# at most what its items' bytes allow (`[HEAP-8]`).

fn main():
    n = 2000000
    total = 0
    for round in 0 .. 3:
        for (i, v) in (round .. n).step_by(2).enumerate():
            total ^= i ^ v
    m = n // 2 - 5
    count: int = (m .. n).iter().len()
    println(total, count)
    sums: Array[int] = []
    for round in (0 .. 3).rev():
        s = 0
        for (i, v) in (round .. n).iter().enumerate():
            s += i + v
        sums.push(s)
    third = 0
    for round in range(0, 3, 2):
        for (i, v) in (round .. n).iter().enumerate():
            third += i * 3 + v
    println(sums[0], sums[2], third)
    ys = [10, 20, 30]
    line: Array[String] = []
    for k, y in ys.enumerate(start=int.MAX - 2):
        top: int = k
        line.push(f"{top} {y}")
    println(line.join(" "))
    zs: Array[int] = [5, 5, 5]
    size = len(zs)
    numbered = 0
    for i, z in zs.iter().enumerate(start=size):
        numbered += i + z - size
    println(size, numbered)
