#$ test: run-pass
#$ rules: CORO-1, CORO-3, CORO-4, GRM-21
#$ profiles: debug, release
#$ stdout: 3 2 1
#$ 4 6
#$ 10 20
#$ 1 1 2 3 5
# Generators as a program uses them (`[CORO-1]`): a countdown, a filter over a view, methods on
# a struct, an infinite sequence cut short by an adapter. Also `ember fmt`'s round trip over
# `tests/run-pass` (D-662: the formatter dropped `gen`).

gen fn countdown(n: int) -> Generator[int]:
    i = n
    while i > 0:
        yield i
        i -= 1

gen fn evens(xs: Span[int]) -> Generator[int]:
    for x in xs:
        if x % 2 == 0:
            yield x

struct Grid:
    cells: Array[int]

    gen fn scaled(self, k: int) -> Generator[int]:
        for c in self.cells:
            yield c * k

gen fn fibonacci() -> Generator[int]:
    a = 1
    b = 1
    while true:
        yield a
        a, b = b, a + b

fn main():
    down = [t for t in countdown(3)]
    println(down[0], down[1], down[2])
    data = [1, 4, 5, 6]
    even = [x for x in evens(data)]
    println(even[0], even[1])
    g = Grid([1, 2])
    big = [x for x in g.scaled(10)]
    println(big[0], big[1])
    fib = [x for x in fibonacci().take(5)]
    println(fib[0], fib[1], fib[2], fib[3], fib[4])
