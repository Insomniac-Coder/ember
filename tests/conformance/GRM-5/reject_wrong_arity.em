#$ test: compile-fail
#$ rules: GRM-5
#$ error[E2020]: `(i64, i64)` has 2 fields, but this destructuring has 3 targets

fn main():
    a, b, c = (1, 2)

