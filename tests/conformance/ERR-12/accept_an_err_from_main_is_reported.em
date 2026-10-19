#$ test: run-pass
#$ rules: ERR-12, FN-8, ERR-2
#$ profiles: debug, development, release
#$ stdout: 5
#$ stderr: error: 7 is odd
#$ exit: 1
# D-549 — an `Err` returned from `main`, here through `?`, prints
# `error: <Display>` to standard error and exits with status 1; an `E` that is
# `Display` but not `Error` prints its message alone.

fn half(n: int) -> Result[int, String]:
    if n % 2 != 0:
        return Err(f"{n} is odd")
    return Ok(n // 2)

fn main() -> Result[void, String]:
    println(half(10)?)
    println(half(7)?)
    println("not reached")
