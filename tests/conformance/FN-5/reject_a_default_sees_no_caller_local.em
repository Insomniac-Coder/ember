#$ test: compile-fail
#$ rules: FN-5
#$ error[E1010]: cannot find `secret` in this scope
# `[FN-5]` — a default is resolved where the function is declared: the
# caller's locals are not in its scope.

fn hidden(x: int = secret) -> int:
    return x

fn main():
    secret = 5
    println(hidden())
