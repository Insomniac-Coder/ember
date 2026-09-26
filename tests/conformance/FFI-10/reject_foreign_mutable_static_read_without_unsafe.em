#$ test: compile-fail
#$ rules: FFI-10, STA-1
#$ error[E5002]: reading a foreign static requires `unsafe`

unsafe extern "C":
    static mut changing_counter: i32

fn main():
    println(changing_counter)
