#$ test: run-pass
#$ rules: ERR-1, STD-9, LEX-19
#$ stdout: Failure(code=4)
#$ Failure(code=4)
#$ Failure(code=4) Failure(code=4)
#$ Bad
# `[ERR-1]` — printing an error needs `Display`, or `Debug` as `println` falls back (`[STD-9]`,
# `[LEX-19]`): an `E` with `Debug` and no `Display` prints by its `Debug` text, in `println`, an
# f-string and a held print; it is only `main`'s report (`[FN-8]`) that needs `Display`.

@derive(Debug)
struct Failure:
    code: int

@derive(Debug)
enum Kind:
    Bad

fn fail() -> Result[int, Failure]:
    return Err(Failure(4))

fn main():
    match fail():
        Ok(_):
            println("ok")
        Err(e):
            println(e)
            println(f"{e}")
            println(e, f"{e!r}")
    println(Kind.Bad)
