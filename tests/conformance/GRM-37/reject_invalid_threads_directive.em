#! threads elsewhere
#$ test: compile-fail
#$ rules: GRM-37
#$ profiles: debug
#$ error[E0104]: `#! threads` must be `main`, `any`, or `creator`

fn main():
    pass
