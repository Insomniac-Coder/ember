#$ test: run-pass
#$ rules: ERR-2, ERR-7
#$ profiles: debug, development, release
#$ stdout: 7
#$ low -4
#$ same -1
#$ generic -7
#$ moved 3
# `[ERR-2]` — `e?` on a `Result[T, E]` in a function returning `Result[U, F]`
# returns `Err(F.from(err))`: here `HighError`'s `From[LowError]`. One error
# type to itself is a move with no call, in a generic body too, where `F` is
# `E` and `E.from` is the prelude's identity (`[ERR-7]`).

struct LowError:
    code: int

enum HighError:
    Low(LowError)
    Other

extend HighError implements From[LowError]:
    fn from(owned value: LowError) -> HighError:
        return HighError.Low(value)

fn low(n: int) -> Result[int, LowError]:
    if n < 0:
        return Err(LowError(n))
    return Ok(n * 2)

fn high(n: int) -> Result[int, HighError]:
    v = low(n)?
    return Ok(v + 1)

fn same(n: int) -> Result[int, LowError]:
    v = low(n)?
    return Ok(v)

fn generic[E, F: From[E]](owned r: Result[int, E]) -> Result[int, F]:
    v = r?
    return Ok(v)

fn report(r: Result[int, HighError], label: str):
    match r:
        Ok(v):
            println(v)
        Err(HighError.Low(e)):
            println(f"{label} {e.code}")
        Err(HighError.Other):
            println("other")

fn main():
    report(high(3), "low")
    report(high(-4), "low")
    match same(-1):
        Ok(v):
            println(v)
        Err(e):
            println(f"same {e.code}")
    report(generic(low(-7)), "generic")
    kept: Result[int, LowError] = generic(low(-3))
    match kept:
        Ok(v):
            println(v)
        Err(e):
            println(f"moved {-e.code}")
