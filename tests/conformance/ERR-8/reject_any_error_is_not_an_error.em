#$ test: compile-fail
#$ rules: ERR-8, ERR-1
# `[ERR-8]` — `AnyError` does not implement `Error`: otherwise its `From[E]`
# for every `E: Error` would give it a second `From[AnyError]` beside the
# prelude's own (`[ERR-7]`). And converting into it needs `E: Error`
# (`[ERR-1]`): an `int` error does not convert.

fn report[E: Error](e: E):
    println(f"{e}")

fn number(n: int) -> Result[int, int]:
    return Err(n)

fn any(n: int) -> Result[int, AnyError]:
    v = number(n)?    #$ error[E2040]: `AnyError` does not implement `From[i64]`
    return Ok(v)

fn main():
    r: Result[int, AnyError] = any(1)
    match r:
        Ok(v):
            println(v)
        Err(e):
            report(e)    #$ error[E2040]: `AnyError` does not implement `Error`
