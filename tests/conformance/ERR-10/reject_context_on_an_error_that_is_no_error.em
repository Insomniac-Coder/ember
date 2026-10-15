#$ test: compile-fail
#$ rules: ERR-10, ERR-1
# `[ERR-10]` — `context` needs the error to convert into an `AnyError`
# (`AnyError: From[E]`), which an `int` error does not: only an `E: Error`
# does (`[ERR-1]`).

fn number(n: int) -> Result[int, int]:
    return Err(n)

fn main():
    r = number(1).context("reading")    #$ error[E2040]: `AnyError` does not implement `From[i64]`
    println(r.is_ok())
