#$ test: compile-fail
#$ rules: WK-1
#$ error[E2020]: `Weak` requires a class handle type

fn main():
    invalid = Weak(1)
