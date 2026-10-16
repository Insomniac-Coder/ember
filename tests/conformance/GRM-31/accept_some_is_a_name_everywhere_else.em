#$ test: run-pass
#$ rules: GRM-31, LEX-15
#$ stdout: 7 some
# `[GRM-31]` — `some` makes a `some` type only in a type, before the interface it names; it is no
# reserved word (`[LEX-15]`), so a function, a local and a field may still be called `some`.

struct Pick:
    some: String

fn some(x: int) -> int:
    return x + 1

fn main():
    some_value = some(3)
    some = 3
    p = Pick(some="some")
    println(some_value + some, p.some)
