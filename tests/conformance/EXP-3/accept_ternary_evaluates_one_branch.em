#$ test: run-pass
#$ rules: EXP-3
#$ profiles: debug, release, shipping
#$ stdout: big big
#$ then
#$ 1
#$ 7 2.5
#$ -1 0 1
# `x if c else y` evaluates `c`, then exactly one branch. It is
# right-associative, and an untyped literal branch takes the other's type.

fn loud(label: str, value: int) -> int:
    println(label)
    return value

fn sign(x: int) -> int:
    return -1 if x < 0 else 0 if x == 0 else 1

fn main():
    q = 5
    label = "big" if q > 3 else "small"
    print("small" if q < 3 else "big", "")
    println(label)
    picked = loud("then", 1) if q > 3 else loud("else", 2)
    println(picked)
    narrow: i32 = 7
    wide = narrow if q > 0 else 0
    half = 2.5 if q > 0 else 1
    println(wide, half)
    println(sign(-4), sign(0), sign(9))
