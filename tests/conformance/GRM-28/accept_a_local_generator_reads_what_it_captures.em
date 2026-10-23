#$ test: run-pass
#$ rules: GRM-28, GRM-21, CLO-12, CLO-2, CORO-1
#$ profiles: debug, development, release
#$ stdout: 10 11 12
#$ 20 21
## `[GRM-28]`, `[GRM-21]` (ODR-268) — a `gen fn` is legal in a block, as `fn` is: a named closure
## (`[CLO-12]`) reading `base` from the function around it. Each call borrows what it captures
## (`[CLO-2]`: shared, since the body only reads it) for as long as that generator lives, so
## `base` may change between two generators of it.

fn main():
    base = 10
    gen fn count(n: int) -> Generator[int]:
        for i in 0..n:
            yield base + i
    got = [v for v in count(3)]
    println(got[0], got[1], got[2])
    base = 20
    again = [v for v in count(2)]
    println(again[0], again[1])
