#$ test: compile-fail
#$ rules: LT-45
#$ help: s: Array[int]
## design rules: R7
## R7: a struct that holds a view lives no longer than what its view fields point into, with or without
## `@view`; returning it past the list it views is refused (E3063).

struct Holder:
    s: Span[int]

fn make() -> Holder:
    xs = [1, 2, 3]
    return Holder(s=xs.as_span())    #$ error[E3063]: the `Holder` returned here holds a view of `xs` (its field `s`), and `xs` ends when `make` returns; let `Holder` keep its own numbers: write `s: Array[int]` in `Holder` and `return Holder(s=xs)` in place of this line

fn main():
    println(make().s.len())
