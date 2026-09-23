#$ test: compile-fail
#$ rules: FN-10
#$ error[E2182]: this function returns `i64` but can reach the end of its body
# No return type other than `void` and `Result[void, E]` has an implicit
# value, so a body that can reach its end is rejected (ODR-023).

fn sign(x: int) -> int:
    if x > 0:
        return 1
    elif x < 0:
        return -1

fn main():
    println(sign(0))
