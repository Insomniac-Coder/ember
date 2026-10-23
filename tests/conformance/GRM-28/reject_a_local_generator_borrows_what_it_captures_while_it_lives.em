#$ test: compile-fail
#$ rules: GRM-28, CLO-2, BRW-2, CORO-6
## `[GRM-28]` (ODR-268) — a generator made of a local generator borrows what it captures for as long
## as it lives (`[BRW-2]`): mutably what its body changes, shared what it reads, so neither may be
## written while the generator is still to be resumed. Its own frame's locals are as any
## generator's (`[CORO-6]`): an inner generator capturing one may not be held across a `yield`.

fn main():
    total = 0
    gen fn running(owned xs: Array[int]) -> Generator[int]:
        for x in owned xs:
            total += x
            yield total
    g = running([1, 2])
    total = 5    #$ error[E3021]: `total` cannot be written while it is borrowed
    println(g.next())
    base = 2
    gen fn reading(n: int) -> Generator[int]:
        yield base * n
    h = reading(3)
    base = 4    #$ error[E3021]: `base` cannot be written while it is borrowed
    println(h.next())
    gen fn outer(n: int) -> Generator[int]:
        gen fn inner(m: int) -> Generator[int]:
            yield m + n
        for v in inner(1):
            yield v    #$ error[E2221]: a borrow of `n`, a local of this generator, is held across this `yield`
    for v in outer(1):
        println(v)
