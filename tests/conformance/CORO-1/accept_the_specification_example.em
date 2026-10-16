#$ test: run-pass
#$ rules: CORO-1, CORO-3, CORO-4, GRM-21
#$ profiles: debug, release
#$ stdout: 3
#$ 2
#$ 1
#$ 6
# `[CORO-1]` — the specification's own example: a generator over its own counter, and one
# over a view its caller lends it, iterated by `for` and summed by an adapter (`[CORO-3]`).

gen fn countdown(n: int) -> Generator[int]:
    i = n
    while i > 0:
        yield i
        i -= 1

gen fn evens(xs: Span[int]) -> Generator[int]:
    for x in xs:
        if x % 2 == 0:
            yield x

fn main():
    for t in countdown(3):
        println(t)
    data = [1, 2, 3, 4]
    total = evens(data).sum()
    println(total)
