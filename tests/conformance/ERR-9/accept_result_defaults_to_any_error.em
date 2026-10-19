#$ test: run-pass
#$ rules: ERR-9, ERR-1
#$ profiles: debug, development, release
#$ stdout: 6
#$ disk 2
#$ -1
# `[ERR-9]` — `Result[T]` is `Result[T, AnyError]`: a function that can fail
# with any error. Neither parameter of `Result` is bounded (`[ERR-1]`):
# `Result[int, int]` is a `Result`.

@derive(Error)
@error("disk {code}")
struct Disk:
    code: int

fn disk(n: int) -> Result[int, Disk]:
    if n % 2 == 0:
        return Err(Disk(n))
    return Ok(n)

fn load(n: int) -> Result[int]:
    v = disk(n)?
    return Ok(v * 2)

fn same(owned r: Result[int]) -> Result[int, AnyError]:
    return r

fn plain(n: int) -> Result[int, int]:
    return Err(-n)

fn main():
    match same(load(3)):
        Ok(v):
            println(v)
        Err(e):
            println(e)
    match load(2):
        Ok(v):
            println(v)
        Err(e):
            println(e)
    match plain(1):
        Ok(v):
            println(v)
        Err(e):
            println(e)
