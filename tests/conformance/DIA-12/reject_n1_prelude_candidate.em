#$ rules: DIA-12
#$ test: compile-fail
#$ error[E1010]: cannot find `Ordd` in this scope
#$ help: did you mean `Ord`?
fn main():
    Ordd()
