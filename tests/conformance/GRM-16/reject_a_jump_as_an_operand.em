#$ test: compile-fail
#$ rules: GRM-16
# `[GRM-16]` — a jump is a whole expression statement or the whole of a
# lambda body, a match arm or a conditional branch; as an operand it is
# `E0107`.

fn g(b: int) -> int:
    return 1 + return 2     #$ error[E0107]: a jump expression may not be an operand

fn main():
    println(g(1))
