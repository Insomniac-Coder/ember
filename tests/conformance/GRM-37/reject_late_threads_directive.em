#$ test: compile-fail
#$ rules: GRM-37
#$ profiles: debug
#$ error[E0104]: a directive must appear before imports and declarations

fn main():
    pass

#! threads main
