#$ rules: DIA-12
#$ test: compile-fail
#$ error[E1010]: cannot find `pritn` in this scope
#$ help: did you mean `prtin`?
#$ help: did you mean `print`?
from support.text import print
from support.io import print as prtin

fn main():
    pritn(1)
