#$ test: run-fail
#$ rules: STD-19, STD-26, TYP-8
#$ profiles: debug, release, shipping
#$ stdout:
#$ 9223372036854775806
#$ 9223372036854775807
#$ panics: integer overflow in `+`
# `[STD-19]` — `enumerate` numbers item `k` as `start + k`: the item numbered
# `int.MAX` is given, and the one after it panics, as a `+` past the top does.

fn main():
    xs: Array[int] = [1, 2, 3, 4]
    it = xs.iter().enumerate(start=int.MAX - 1)
    for (i, _) in it:
        println(i)
