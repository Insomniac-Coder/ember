#$ rules: DIA-12
#$ test: compile-fail
#$ error[E1010]: cannot find `pritn` in this scope
#$ help: did you mean `prtin`?
#$ help: did you mean `print`?
#$ help: did you mean `printz`?
from support.io import print

fn printz(value: i32) -> i32:
    return value

fn main():
    prtin = 1
    pritn(2)
