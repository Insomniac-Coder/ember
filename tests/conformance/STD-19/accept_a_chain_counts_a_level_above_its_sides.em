#$ test: run-pass
#$ rules: STD-19, IFC-4
#$ stdout: 5 6
#$ stdout: 36893488147419103230 680564733841876926926749214863536422912
#$ stdout: 5 6
#$ stdout: [1, 0]
#$ stdout: [(36893488147419103229, 18446744073709551614)]
#$ stdout: [4, 5, 0, 1, 2]
#$ profiles: debug, release
# G8-4 (the owner's design, 0.9.10) — a chain counts the two sides' items together, stepping up
# by need: two `int` counts stay an `int` (stored items and small ranges cannot come near its
# top); otherwise the count is one level above the wider side's, `u64` -> `u128` -> `u256`. In
# generic code it is the two counts together, filled in for each use: `total` gives 5 for two
# arrays and 6 for two `u64`-counted ranges, each through `to_int()`. A chain of two whole `u64`
# ranges holds 36,893,488,147,419,103,230 items, and runs backwards, `take`n and `enumerate`d,
# at once.

from std.core import ExactSizeIterator

fn total[I: ExactSizeIterator, J: ExactSizeIterator[Item = I.Item]](owned a: I, owned b: J) -> int:
    return a.chain(b).len().to_int()

fn first[I: Iterator](owned from: I, n: int) -> Array[I.Item]:
    it = from
    out: Array[I.Item] = []
    while out.len() < n:
        match it.next():
            Some(x):
                out.push(x)
            None:
                return out
    return out

fn main():
    xs: Array[int] = [1, 2, 3]
    ys: Array[int] = [4, 5]
    small = 0 .. 3
    a: int = xs.iter().chain(ys.iter()).len()
    b: u128 = xs.iter().copied().chain(small.iter()).len()
    println(a, b)
    big = 0 as u64 .. u64.MAX
    every = 0 as u128 ..= u128.MAX
    c: u128 = big.iter().chain(big.iter()).len()
    d: u256 = every.iter().chain(every.iter()).len()
    println(c, d)
    println(total(xs.iter(), ys.iter()), total(small.iter(), small.iter()))
    println(first(big.iter().chain(big.iter()).take(2).rev(), 2))
    println(first(big.iter().chain(big.iter()).enumerate().rev(), 1))
    println(first(ys.iter().copied().chain(small.iter()), 5))
