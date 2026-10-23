#$ test: compile-fail
#$ rules: GRM-28, GRM-21, CORO-2, CORO-10, LNT-4
## `[GRM-28]` (ODR-268) — a local generator's body is checked where it is written, once: each
## mistake is reported there and by no generator made of it (`bad` is called twice). As any
## `gen fn` it needs `Generator[Y]` (`E2020`), cannot be `extern` (`E2222`), and one that never
## yields is linted (`L2004`); a lambda inside it cannot yield for it (`E2220`).

fn main():
    gen fn bad(n: int) -> Generator[int]:
        yield n + "a"    #$ error[E2020]: `+` cannot be applied to `i64` and `str`
    for v in bad(1):
        println(v)
    for v in bad(2):
        println(v)
    gen fn stray(n: int) -> Generator[int]:
        f = fn(x: int):
            yield x    #$ error[E2220]: `yield` outside a `gen fn`
        yield n
    for v in stray(1):
        println(v)
    gen fn plain(n: int) -> int:    #$ error[E2020]: a `gen fn` returns `Generator[Y]` or `Generator[Y, R]`
        return n
    extern "C" gen fn exported(n: i32) -> Generator[i32]:    #$ error[E2222]: a `gen fn` cannot be `extern` or `@export`ed
        yield n
    gen fn none(n: int) -> Generator[int]:    #$ warning[L2004]: `gen fn none` never yields
        println(n)
    for v in none(1):
        println(v)
