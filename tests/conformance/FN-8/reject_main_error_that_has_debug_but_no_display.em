#$ test: compile-fail
#$ rules: FN-8, ERR-1
#$ help: implement `Display` for `Failure`
# `[FN-8]` — `fn main() -> Result[void, E]` needs `E: needs Display`: an `Err` is printed with
# `Display`, so an `E` that only has `Debug` is `E2040`, an unmet bound.

struct Failure:
    code: int

fn main() -> Result[void, Failure]: #$ error[E2040]: `Failure` does not implement `Display`, which reporting `main`'s error needs
    return Err(Failure(code=1))
