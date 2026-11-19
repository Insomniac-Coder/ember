#$ test: compile-fail
#$ rules: ERR-1, ERR-2, ERR-8
# `[ERR-1]` — converting an error into `AnyError` takes an `E: needs Error`: a type that only has
# `Display` and `Debug` is not one, so `?` into `AnyError` is `E2040`; walking an error's causes
# (`source`) needs `Error` too.

@derive(Debug)
struct Plain:
    n: int

fn one() -> Result[int, Plain]:
    return Err(Plain(1))

fn two() -> Result[int]:
    v = one()?    #$ error[E2040]: `?` cannot carry the failure of a `Result[i64, Plain]` out of a function returning `Result[i64, AnyError]`
    return Ok(v)

fn main():
    p = Plain(2)
    println(p.source())    #$ error[E2072]: `Plain` has no method named `source`
