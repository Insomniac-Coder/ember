#$ test: run-pass
#$ rules: STD-19, STD-26, TYP-8
#$ profiles: debug, release, shipping
#$ stdout:
#$ 9223372036854775806
#$ 9223372036854775807
#$ 9223372036854775808
#$ 9223372036854775809
# `[STD-19]` (the owner's ruling of 2026-10-06, replacing G8-4 decision F) — `enumerate` numbers
# item `k` as `start + k`, in a type that holds every number it can give: from `int.MAX - 1`,
# over a list of four items, the numbers pass `int`'s top, so they are `u64`s. The iterator is
# kept in a variable and walked by `Enumerate.next`.

fn main():
    xs: Array[int] = [1, 2, 3, 4]
    it = xs.iter().enumerate(start=int.MAX - 1)
    for (i, _) in it:
        println(i)
