#$ test: run-fail
#$ rules: RT-12
#$ profiles: debug, release
#$ panics: stack overflow in deep
# `[RT-12]` — a recursion that never ends overflows its stack into the guard page, and the process
# aborts saying so and naming the function, in every profile. The result is used after the call
# (through a small array on the frame), so no C compiler turns the recursion into a loop.

fn deep(n: int) -> int:
    near = [n, n + 1, n + 2, n + 3]
    r = deep(n + 1)
    return r + near[r % 4]

fn main():
    println(deep(0))
