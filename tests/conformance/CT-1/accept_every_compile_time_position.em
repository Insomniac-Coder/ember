#$ test: run-pass
#$ rules: CT-1, CT-6, CT-7, TYP-41, STA-3, GRM-32
#$ stdout: 55 120 [0, 0, 0] 3628800 6 ok
# `[CT-1]` — what is evaluated while compiling: `comptime(e)`, a `comptime:` block, a `const`
# initialiser, an array length and a `const` generic argument, and a `static` initialiser. Each
# calls an ordinary function here, which allocates (`Alloc` is allowed) and may panic.

fn fib(n: int) -> int:
    seen: Array[int] = [0, 1]
    while seen.len() <= n:
        seen.push(seen[seen.len() - 1] + seen[seen.len() - 2])
    return seen[n]

fn factorial(n: int) -> int:
    if n < 0:
        panic("negative")
    return 1 if n == 0 else n * factorial(n - 1)

fn side() -> int:
    return 3

fn zeros[const N: int]() -> [int; N]:
    return [0; N]

const FIB: int = fib(10)
static FACT: int = factorial(10)

fn main():
    comptime:
        assert(factorial(5) == 120)
    row: [int; side()] = zeros[side()]()
    println(FIB, comptime(factorial(5)), row, FACT, side() * 2, "ok")
