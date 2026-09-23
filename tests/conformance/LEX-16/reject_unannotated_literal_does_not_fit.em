#$ test: compile-fail
#$ rules: LEX-16
#$ error[E2010]: does not fit
# An unannotated literal is `int`, and a value that does not fit it is E2010.

fn main():
    big = 99999999999999999999
    println(big)
