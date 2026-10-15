#$ test: run-pass
#$ rules: ERR-11, ERR-2, EFF-5
#$ profiles: debug, release, shipping
#$ stdout: true
#$ false
# `[ERR-11]` — a concrete error type moves through `?` without allocating, so
# `@noalloc` code that names its error types may use `?`.

@derive(Error)
@error("disk {code}")
struct Disk:
    code: int

@noalloc
fn disk(n: int) -> Result[int, Disk]:
    if n < 0:
        return Err(Disk(n))
    return Ok(n)

@noalloc
fn read(n: int) -> Result[int, Disk]:
    v = disk(n)?
    return Ok(v + 1)

fn main():
    println(read(1).is_ok())
    println(read(-1).is_ok())
