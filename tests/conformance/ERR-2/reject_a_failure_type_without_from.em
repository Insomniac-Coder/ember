#$ test: compile-fail
#$ rules: ERR-2, ERR-1
#$ help: implement `From[LowError]` for `Other`
# `[ERR-2]` — `?` into another error type needs that type's `From[E]`
# (`[ERR-1]`): `Other` has none for `LowError`.

struct LowError:
    code: int

struct Other:
    code: int

fn low(n: int) -> Result[int, LowError]:
    return Err(LowError(n))

fn high(n: int) -> Result[int, Other]:
    v = low(n)?    #$ error[E2040]: `Other` does not implement `From[LowError]`
    return Ok(v)

fn main():
    println(high(1).is_ok())
