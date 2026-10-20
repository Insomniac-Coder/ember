#$ perf: scalar
## `[TST-28]` — `bench/programs/a10_recursion.em` and its twin, gated.

fn fib(n: int) -> int:
    if n < 2:
        return n
    return fib(n - 1) + fib(n - 2)

fn main():
    println(fib(38))
