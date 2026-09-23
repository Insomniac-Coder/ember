#$ rules: DIA-12
#$ test: compile-fail
#$ error[E1010]: cannot find type `Dto` in this scope
#$ error[E1010]: cannot find type `Vetc` in this scope
#$ help: did you mean `Dot`?
#$ help: did you mean `Vect`?
from support.models import Point as Dot, Vector as Vect

fn takes(value: Dto):
    pass

fn takes_vector(value: Vetc[i32]):
    pass

fn main():
    pass
