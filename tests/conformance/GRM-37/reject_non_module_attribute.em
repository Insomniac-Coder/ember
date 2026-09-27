#$ test: compile-fail
#$ rules: GRM-37, ATT-1
#$ profiles: debug
#$ error[E0104]: `@inline` does not apply to a module

#! module inline

fn main():
    pass
