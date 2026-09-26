#$ test: compile-fail
#$ rules: TYP-7, UNS-1
#$ profiles: debug
#$ error[E3100]: casting between raw pointer types requires `unsafe`

fn cast(p: *mut i32):
    q: *mut u8 = p as *mut u8

fn main():
    pass
