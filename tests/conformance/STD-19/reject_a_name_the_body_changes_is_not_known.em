#$ test: compile-fail
#$ rules: STD-19, TYP-4
# `[STD-19]` (the owner's rulings of 2026-10-06) — the known-numbers rule follows a name only
# when it is set once and never changed. Each of these functions changes `m` (or `ys`) somewhere:
# later in the loop, through a `mut` argument, through a `mut self` call (an interface's method),
# in a closure, or by `push`. So nothing is known of it, `(m .. 10)` may have more items than an `int` numbers, and
# the numbers are `u64`s, which `total += i` cannot add to an `int`.

fn bump(mut n: int):
    n -= 1000

interface Lower:
    fn lower(mut self)

extend int implements Lower:
    fn lower(mut self):
        self -= 1000

fn by_assignment_later_in_the_loop() -> int:
    m = -5
    total = 0
    for round in 0 .. 3:
        for i, x in (m .. 10).iter().enumerate():
            total += i              #$ error[E2020]: this is `u64`
        m = int.MIN
    return total

fn by_a_mut_argument() -> int:
    m = -5
    total = 0
    for i, x in (m .. 10).iter().enumerate():
        total += i                  #$ error[E2020]: this is `u64`
    bump(m)
    return total

fn by_a_mut_self_call() -> int:
    m = -5
    total = 0
    for i, x in (m .. 10).iter().enumerate():
        total += i                  #$ error[E2020]: this is `u64`
    m.lower()
    return total

fn by_a_closure() -> int:
    m = -5
    lower = fn():
        m -= 1000
    total = 0
    for i, x in (m .. 10).iter().enumerate():
        total += i                  #$ error[E2020]: this is `u64`
    lower()
    return total

fn by_push() -> int:
    ys = [10, 20, 30]
    ys.push(40)
    total = 0
    for k, y in ys.enumerate(start=int.MAX - 2):
        total += k                  #$ error[E2020]: this is `u64`
    return total

fn main():
    println(by_assignment_later_in_the_loop(), by_a_mut_argument(), by_a_mut_self_call(), by_a_closure(), by_push())
