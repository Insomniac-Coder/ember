#$ test: compile-fail
#$ rules: WK-1
#$ error[E2020]: `Weak` requires a class handle or `Shared[T]` owner type

fn main():
    invalid = Weak(1)
