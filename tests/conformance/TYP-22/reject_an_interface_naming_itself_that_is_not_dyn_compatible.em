#$ test: compile-fail
#$ rules: TYP-22, ERR-3
# ADR-186 — an interface may name itself as `dyn` in its own methods
# (`[ERR-3]`'s `Error.source` returns `Option[ref dyn Error]`), and it must
# then be `dyn`-compatible once its methods are known.

interface Node:    #$ error[E2050]: interface `Node` names itself in its methods, but is not `dyn`-compatible
    fn copy(self) -> Self
    fn next(self) -> Option[ref dyn Node]

fn main():
    println(1)
