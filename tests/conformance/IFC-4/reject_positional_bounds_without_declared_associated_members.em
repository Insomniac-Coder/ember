#$ test: compile-fail
#$ rules: IFC-4, TYP-17
# D-540: resolving positional projections does not invent an associated member
# or grant one to an unbounded parameter.

interface Source:
    type Item

interface Build[T]:
    fn make() -> Self

fn absent[S: Source, C: Build[S.Absent]]():    #$ error[E2040]
    pass

fn unbounded[S, C: Build[S.Item]]():    #$ error[E2040]
    pass

fn main():
    pass
