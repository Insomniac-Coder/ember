#$ test: compile-fail
#$ rules: IFC-4, TYP-17
# D-540: resolving positional projections does not invent an associated member
# or grant one to an unbounded parameter.

interface Source:
    type Item

interface Build[T]:
    static fn make() -> Self

[S: needs Source, C: needs Build[S.Absent]]    #$ error[E2040]
fn absent():
    pass

[S, C: needs Build[S.Item]]    #$ error[E2040]
fn unbounded():
    pass

fn main():
    pass
