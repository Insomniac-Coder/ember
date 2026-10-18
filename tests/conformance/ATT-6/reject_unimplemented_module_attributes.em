#$ test: compile-fail
#$ rules: ATT-6, GRM-37
#$ profiles: debug
#$ error[E0900]: `@prelude` is not implemented yet

#! module prelude

fn main():
    pass
