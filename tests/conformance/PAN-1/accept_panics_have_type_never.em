#$ test: run-pass
#$ rules: PAN-1, FN-10, MOD-5
#$ profiles: debug, release, shipping
#$ stdout: 1
#$ positive
#$ 4
# `panic(msg)`, `todo()` and `unreachable()` have type `Never`: they stand
# where any value is expected, and a value function may end in one.

fn sign(x: int) -> int:
    if x > 0:
        return 1
    if x < 0:
        return -1
    unreachable()

fn describe(x: int) -> str:
    return "positive" if x > 0 else panic("not positive")

fn later() -> int:
    todo()

fn main():
    println(sign(5))
    println(describe(3))
    doubled = 2 if sign(1) > 0 else later()
    println(doubled * 2)
