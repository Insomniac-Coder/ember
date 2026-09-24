#$ test: run-pass
#$ rules: GRM-16
#$ stdout:
#$ 11 7
#$ 4 -1
#$ 6
# `[GRM-16]` — a jump is an expression of type `!`, and may be the whole of a
# conditional-expression branch: the other branch gives the value.

fn f(b: bool) -> int:
    v = 1 if b else return 7
    return v + 10

fn first_even(xs: Array[int]) -> int:
    for x in xs:
        half = x // 2 if x % 2 == 0 else continue
        return half
    return -1

fn main():
    println(f(true), f(false))
    println(first_even([3, 5, 8, 9]), first_even([1]))
    total = 0
    for i in 0..10:
        step = i if i < 4 else break
        total += step
    println(total)
