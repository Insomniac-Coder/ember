#$ test: run-pass
#$ rules: STD-19, STD-26, CTL-3b
#$ stdout: [(18446744073709551611, 9223372036854775806), (18446744073709551610, 9223372036854775805)]
#$ stdout: for 18446744073709551611 9223372036854775806
#$ stdout: for 18446744073709551610 9223372036854775805
#$ stdout: [(340282366920938463463374607431768211455, 340282366920938463463374607431768211455)]
#$ stdout: [(7, 30), (6, 20), (5, 10)]
#$ stdout: [(9223372036854775806, 0), (9223372036854775807, 1), (9223372036854775808, 2)]
#$ profiles: debug, release
# G8-4 (the owner's design, 0.9.10) — `enumerate` numbers a range of 64-bit numbers in `i128` and
# one of 128-bit numbers in `i256`, so every item has its number, from a start that may be
# negative. The owner's example: the whole `int` range numbered from -3 and walked backwards,
# where one number goes from 18,446,744,073,709,551,611 down to -3, which no 64-bit type holds.
# Stored items still number in `int`. The ranges are kept in variables, where their numbers are
# not visible (the visible-numbers rule picks a smaller type where they are).

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
    whole = int.MIN .. int.MAX
    println(first(whole.iter().enumerate(-3).rev(), 2))
    n = 0
    for i, x in whole.iter().enumerate(-3).rev():
        println("for", i, x)
        n += 1
        if n == 2:
            break
    every = 0 as u128 ..= u128.MAX
    println(first(every.iter().enumerate().rev(), 1))
    xs: Array[int] = [10, 20, 30]
    println(first(xs.iter().copied().enumerate(5).rev(), 3))
    three = 0 .. 3
    println(first(three.iter().enumerate(int.MAX - 1), 3))
