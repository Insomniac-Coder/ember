#$ test: compile-fail
#$ rules: TYP-7
#$ profiles: debug
#$ error[E2020]: `null[P]()` takes one raw-pointer type argument and no values

fn main():
    pointer = null()
