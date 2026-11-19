#$ rules: DIA-12, DIA-24
#$ test: compile-fail
#$ error[E1010]: cannot find `pritn` in this scope
#$ help: the nearest names are `prin`, `print` and `prit`
#$ not-help: `prtin`
from support.suggestions import prin, print, prit, prtin

fn main():
    pritn()
