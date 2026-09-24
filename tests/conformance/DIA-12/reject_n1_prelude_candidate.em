#$ rules: DIA-12
#$ test: compile-fail
#$ error[E1010]: cannot find `Rangee` in this scope
#$ help: did you mean `Range`?
## A prelude name is a candidate. A value is only offered a name that is one:
## `Range`, a struct with a constructor, not an interface such as `Ord`.
fn main():
    r = Rangee(start=0, end=3)
