#$ test: compile-fail
#$ rules: CORO-2
#$ profiles: debug
# `[CORO-2]` — `yield` outside a `gen fn` is `E2220`, and a lambda inside a generator is a
# function of its own.

fn plain() -> int:
    yield 1    #$ error[E2220]: `yield` outside a `gen fn`
    return 2

gen fn outer() -> Generator[int]:
    _f = fn(x: int) => yield x    #$ error[E2220]: `yield` outside a `gen fn`
    yield 3

fn main():
    println(plain())
    for x in outer():
        println(x)
