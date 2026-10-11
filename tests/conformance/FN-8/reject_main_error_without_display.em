#$ test: compile-fail
#$ rules: FN-8, STR-5
# `main`'s error must be `Display` (`[FN-8]`), to be printed when it returns
# one; a type that opts out of `Debug` has no text at all.

@no_derive(Debug)
struct Failure:
    code: int

fn main() -> Result[void, Failure]: #$ error[E2040]: reporting `main`'s error
    return Err(Failure(code=1))
