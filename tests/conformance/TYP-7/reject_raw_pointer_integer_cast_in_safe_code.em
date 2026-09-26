#$ test: compile-fail
#$ rules: TYP-7, UNS-1
#$ profiles: debug
#$ error[E3100]: casting between a raw pointer and an integer requires `unsafe`

fn main():
    p: *mut i32 = 0 as *mut i32
