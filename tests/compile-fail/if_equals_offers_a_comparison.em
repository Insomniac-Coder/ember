#$ test: compile-fail
#$ rules: GRM-19
#$ error[E2036]: this pattern always matches
#$ help: did you mean `x == 5`?
# `[GRM-19]` — a condition whose pattern is a bare name always matches; the
# first help is the comparison the writer most likely meant.

fn main():
    x = 3
    if x = 5:
        println(x)
