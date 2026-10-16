#$ test: compile-fail
#$ rules: CORO-5
#$ profiles: debug
# `[CORO-5]` — a frame's size is a compile-time constant, so a generator that holds a frame of
# its own across a `yield` (a recursive one, here through another) has none (`E2200`); the inner
# generator is boxed instead.

gen fn walk(n: int) -> Generator[int]:    #$ error[E2200]: holds its own frame across a `yield`
    if n > 0:
        for x in walk(n - 1):
            yield x
    yield n

gen fn ping(n: int) -> Generator[int]:    #$ error[E2200]: holds its own frame across a `yield`
    for x in pong(n):
        yield x

gen fn pong(n: int) -> Generator[int]:    #$ error[E2200]: holds its own frame across a `yield`
    if n > 0:
        for x in ping(n - 1):
            yield x

fn main():
    for x in walk(2):
        println(x)
    for x in ping(2):
        println(x)
