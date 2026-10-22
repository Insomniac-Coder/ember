#$ test: run-pass
#$ rules: GRM-28, CLO-12, CLO-2
#$ profiles: debug, development, release
#$ stdout: 10 3
#$ 4
#$ 20
## `[GRM-28]`, `[CLO-12]` — a local function is a named closure, and a closure inside it may call it:
## a local function, a lambda, and a lambda inside a lambda, reaching it through every closure
## between, with the environment it captures (it was `E0900`). One that captures nothing is called
## directly from anywhere.

fn main():
    base = 10
    calls = 0
    fn outer(n: int) -> int:
        if n <= 0:
            return base
        calls += 1
        fn inner(m: int) -> int:
            return outer(m - 1)
        return inner(n)
    println(outer(3), calls)
    fn plain(n: int) -> int:
        helper = fn(x: int) => plain(x - 1) if x > 0 else 0
        return helper(n) + 1
    println(plain(3))
    fn deep(n: int) -> int:
        a = fn(x: int) -> int:
            b = fn(y: int) => deep(y - 1) + base if y > 0 else 0
            return b(x)
        return a(n)
    println(deep(2))
