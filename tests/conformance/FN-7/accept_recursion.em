#$ test: run-pass
#$ rules: FN-7
#$ stdout: 1000 120
## `[FN-7]` — recursion is permitted, direct or mutual.

fn depth(n: int) -> int:
    if n == 0:
        return 0
    return 1 + depth(n - 1)

fn is_even(n: int) -> bool:
    if n == 0:
        return true
    return is_odd(n - 1)

fn is_odd(n: int) -> bool:
    if n == 0:
        return false
    return is_even(n - 1)

fn factorial(n: int) -> int:
    if n <= 1:
        return 1
    return n * factorial(n - 1)

fn main():
    if is_even(10):
        println(depth(1000), factorial(5))
