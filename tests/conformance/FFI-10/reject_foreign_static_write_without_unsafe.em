#$ test: compile-fail
#$ rules: FFI-10, STA-1
#$ error[E5002]: writing a foreign static requires `unsafe`

unsafe extern "C":
    static mut changing_counter: i32

fn main():
    changing_counter = 9
