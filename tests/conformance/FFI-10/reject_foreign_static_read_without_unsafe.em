#$ test: compile-fail
#$ rules: FFI-10
#$ error[E3100]: reading a foreign static requires `unsafe`

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    static raw_counter: i32

fn main():
    println(raw_counter)
