#$ test: compile-fail
#$ rules: TYP-7
#$ profiles: debug
#$ error[E2020]: `ref i32` cannot be cast to `*mut i32` with `as`

fn main():
    value: i32 = 1
    pointer: *mut i32 = (ref value) as *mut i32
