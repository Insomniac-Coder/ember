#$ test: run-pass
#$ rules: GRM-28, GRM-21, CLO-1, CORO-5
#$ profiles: debug, development, release
#$ stdout: 0 2 4
#$ 3
#$ assert-c: contains("5evens5local5plainG")
#$ assert-c: !contains("5evens5local3envG")
## `[GRM-28]` (ODR-268) — a local generator that captures nothing is the generator a module-level
## `gen fn` would be: its frame holds its parameters and nothing for an environment (the form
## without one is the one made). Its name read as a value is the closure calling it (`[CLO-1]`).

fn main():
    gen fn evens(n: int) -> Generator[int]:
        for i in 0..n:
            if i % 2 == 0:
                yield i
    got = [v for v in evens(6)]
    println(got[0], got[1], got[2])
    make = evens
    println(make(6).count())
