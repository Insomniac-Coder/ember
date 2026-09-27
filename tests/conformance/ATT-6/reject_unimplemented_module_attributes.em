#$ test: compile-fail
#$ rules: ATT-6, GRM-37
#$ profiles: debug
#$ error[E0900]: `@deterministic` is not implemented yet
#$ error[E0900]: `@reloadable` is not implemented yet

#! module deterministic
#! module reloadable

fn main():
    pass
