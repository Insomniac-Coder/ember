#$ test: compile-fail
#$ rules: ATT-1, LT-7
#$ profiles: debug
#$ error[E0104]: `@latebound` is not an attribute
#$ help: delete `@latebound`
# `@latebound` was removed in 0.9.9: every callable type now gets fresh
# regions at each call.

fn apply(f: @latebound fn(int) -> int) -> int:
    return f(2)

fn main():
    println(apply(fn(x) => x + 1))
