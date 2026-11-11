#$ test: run-pass
#$ rules: ERR-1
#$ stdout: true true true false
## `[ERR-1]` — `Option[T]` is absence, `Some(v)` or `None`; `Result[T, E]` is `Ok(v)` or `Err(e)`.
## Both are ordinary enums, and neither parameter is bounded: `Result[int, int]` is a `Result`.

fn main():
    good: Result[int, int] = Ok(3)
    bad: Result[int, int] = Err(7)
    some: Option[int] = Some(5)
    none: Option[int] = None
    println(good.is_ok(), bad.is_err(), some.is_some(), none.is_some())
