#$ test: compile-fail
#$ rules: GRM-37
#$ profiles: debug
#$ error[E0104]: `#! threads main` is a directive, and directives come first

fn main():
    pass

#! threads main
