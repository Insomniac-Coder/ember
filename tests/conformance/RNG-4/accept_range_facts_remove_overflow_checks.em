#$ test: run-pass
#$ rules: RNG-4, TYP-8
#$ profiles: debug, release, shipping
#$ stdout: 6765
#$ stdout: 55
#$ assert-c-count: contains("ember_ck_sub_i64(") == 0
#$ assert-c-count: contains("ember_ck_sub_u32(") == 0
#$ assert-c-count: contains("ember_ck_add_i64(") == 1
# `[RNG-4]` — past `if n < 2: return`, `n` is at least 2, so `n - 1` and
# `n - 2` cannot overflow and carry no check; `fib(..) + fib(..)` depends on
# the results and keeps its check. In `countdown`, `i < n` makes `n - i` at
# least 1: no check, where `total + ..` keeps one.

fn fib(n: int) -> int:
    if n < 2:
        return n
    return fib(n - 1) + fib(n - 2)

fn countdown(n: u32) -> u32:
    total: u32 = 0
    for i in 0..n:
        total = total + (n - i)
    return total

fn main():
    println(fib(20))
    println(countdown(10))
