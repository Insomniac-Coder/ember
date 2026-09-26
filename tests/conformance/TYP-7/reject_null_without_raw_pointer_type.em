#$ test: compile-fail
#$ rules: TYP-7
#$ profiles: debug
#$ error[E2020]: `null[P]()` requires a raw-pointer type `P`

fn main():
    value = null[i32]()
