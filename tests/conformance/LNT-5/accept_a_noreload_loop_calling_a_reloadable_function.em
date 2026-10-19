#$ test: run-pass
#$ rules: LNT-5, HR-10a
#$ stdout: 12
# `L2005`: a `@noreload` function calling a reloadable one inside a loop pays the thunk on every
# turn. The call still goes through the thunk, and the program runs. Outside a loop, or from a
# reloadable function, there is no lint.

@noreload
fn total(xs: Array[int]) -> int:
    sum = 0
    for x in xs:
        sum += weigh(x)  #$ warning[L2005]: `@noreload` function `total` calls reloadable `weigh` inside a loop
    return sum + weigh(0)

fn weigh(x: int) -> int:
    return x * 2

fn count(xs: Array[int]) -> int:
    sum = 0
    for x in xs:
        sum += weigh(x)
    return sum

fn main():
    xs = [1, 2, 3]
    println(total(xs) + count(xs) - 12)
