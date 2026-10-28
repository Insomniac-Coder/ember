#$ test: compile-fail
#$ rules: FFI-10, STA-1
#$ error[E5002]: writing a foreign static requires `unsafe`

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    static mut changing_counter: i32

fn main():
    changing_counter = 9
