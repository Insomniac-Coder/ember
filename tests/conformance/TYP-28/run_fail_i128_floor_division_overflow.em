#$ test: run-fail
#$ rules: TYP-28, TYP-8, TYP-1
#$ profiles: debug, release
#$ panics: integer overflow in `//`
# D-272 — `i128`'s least value floor-divided by -1 is not representable and
# panics, naming the operator written.

fn divide(a: i128, b: i128) -> i128:
    return a // b

fn main():
    lo: i128 = -170141183460469231731687303715884105727 - 1
    println(divide(lo, 2))
    println(divide(lo, -1))
