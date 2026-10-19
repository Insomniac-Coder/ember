#$ test: run-pass
#$ rules: CORO-2, CORO-3
#$ profiles: debug, development
#$ stdout: 10 20 30
#$ 7
# `[CORO-2]` — `yield e` suspends the generator and produces `e`, converted to the yielded
# type as an argument would be; a bare `yield` in a `Generator[void]` produces `()`.

gen fn tens() -> Generator[i64]:
    yield 10
    yield 20
    yield 30

gen fn beats(n: int) -> Generator[void]:
    for _ in 0..n:
        yield

fn main():
    xs = [x for x in tens()]
    println(xs[0], xs[1], xs[2])
    count = 0
    for _ in beats(7):
        count += 1
    println(count)
