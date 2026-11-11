#$ test: compile-fail
#$ rules: TYP-22, ERR-3
# ADR-186 — an interface may name itself as `dyn` in its own methods
# (`[ERR-3]`'s `Error.source` returns `Option[ref dyn Error]`), and it must
# then be `dyn`-compatible once its methods are known.

interface Node:    #$ error[E2050]: `Node` cannot be used as `dyn Node`: `copy` returns `Self` by value, and through a `dyn Node` the kind of the result is unknown
    fn copy() -> Self
    fn next() -> Option[ref dyn Node]

fn main():
    println(1)
