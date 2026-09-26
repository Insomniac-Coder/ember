#$ test: compile-fail
#$ rules: TYP-7
#$ profiles: debug
#$ error[E2020]: `ref_to_ptr` requires a shared or mutable reference

fn main():
    pointer = ref_to_ptr(7)
