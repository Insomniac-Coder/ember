#$ test: compile-fail
#$ rules: FFI-10
#$ error[E5002]: reading a foreign static requires `unsafe`

unsafe extern "C":
    static raw_counter: i32

fn main():
    println(raw_counter)
