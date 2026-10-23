#$ test: run-pass
#$ rules: GRM-28, GRM-21, CLO-12, CLO-2
#$ profiles: debug, development, release
#$ stdout: 6
#$ 3
#$ 20 2
#$ 1 5
## `[GRM-28]` (ODR-268) — a closure in the block may call a local generator: it captures what the
## generator captures, as the generator does (`calls` mutably, `base` shared), and holds it while
## it lives, as any closure (`[CLO-2]`); the lambda inside a lambda reaches it through the one
## around it, and one that captures nothing is called directly. Under a later local of a captured
## name, a call still passes what the generator captured (`base`, the loop's being another).

fn main():
    base = 1
    calls = 0
    gen fn upto(n: int) -> Generator[int]:
        calls += 1
        for i in 0..n:
            yield base + i
    gen fn quiet(n: int) -> Generator[int]:
        for i in 0..n:
            yield i
    sum_of = fn(n: int) -> int:
        total = 0
        for v in upto(n):
            total += v
        return total
    println(sum_of(3))
    count_of = fn(n: int) -> int => quiet(n).count()
    println(count_of(3))
    deep = fn(n: int) -> int:
        inner = fn(m: int) -> int => upto(m).count() * 10
        return inner(n)
    println(deep(2), calls)
    for base in 5..6:
        got = [v for v in upto(1)]
        println(got[0], base)
