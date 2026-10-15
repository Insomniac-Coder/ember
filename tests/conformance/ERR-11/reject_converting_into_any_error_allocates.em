#$ test: compile-fail
#$ rules: ERR-11, EFF-5
# `[ERR-11]` — converting an error into an `AnyError` allocates, once, on the
# failure path, and carries `Alloc`: a `@noalloc` function returning
# `Result[T]` cannot `?` a concrete error into it.

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
fn any(n: int) -> Result[int]:
    v = disk(n)?    #$ error[E4001]: `@noalloc` function `any` reaches an allocation
    return Ok(v)

fn main():
    println(any(1).is_ok())
