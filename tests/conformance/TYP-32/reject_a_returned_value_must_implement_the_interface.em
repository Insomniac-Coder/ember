#$ test: compile-fail
#$ rules: TYP-32
#$ error[E2040]: `Blob` does not implement `Display`, which the function's `some` return type names
# `[TYP-32]` — the type a `some I` function returns implements `I`.

struct Blob:
    n: int

fn blob() -> some Display:
    return Blob(n=1)

fn main():
    println(blob())
