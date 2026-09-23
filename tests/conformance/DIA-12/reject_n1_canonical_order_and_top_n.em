#$ rules: DIA-12
#$ test: compile-fail
#$ error[E1010]: cannot find `pritn` in this scope
#$ help: did you mean `prin`?
#$ help: did you mean `print`?
#$ help: did you mean `prit`?
#$ not-help: `prtin`
from support.suggestions import prin, print, prit, prtin

fn main():
    pritn()
