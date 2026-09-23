#$ rules: DIA-12
#$ test: run-pass
from support.models import Point as Dot, Vector as Vect

fn takes(value: Dot):
    pass

fn takes_vector(value: Vect[i32]):
    pass

fn main():
    pass
