#$ test: run-pass
#$ rules: GRM-28, GRM-21, CLO-12, CLO-2, BRW-2
#$ profiles: debug, development, release
#$ stdout: 1 3 6
#$ 6
#$ 16
## `[GRM-28]`, `[CLO-2]` (ODR-268) — a local generator that writes what it captures borrows it
## mutably, for as long as each generator made of it lives (`[BRW-2]`): `total` is read after the
## loop, when the generator is gone, and the next call borrows it again.

fn main():
    total = 0
    gen fn running(xs: Array[int]) -> Generator[int]:
        for x in xs:
            total += x
            yield total
    seen = [v for v in running([1, 2, 3])]
    println(seen[0], seen[1], seen[2])
    println(total)
    for _ in running([10]):
        pass
    println(total)
