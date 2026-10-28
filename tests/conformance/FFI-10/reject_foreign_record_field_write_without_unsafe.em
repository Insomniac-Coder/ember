#$ test: compile-fail
#$ rules: FFI-10, STA-1
#$ error[E3100]: writing a foreign static requires `unsafe`

@derive(Copy)
struct Pair:
    left: i32

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    static mut foreign_pair: Pair

fn main():
    foreign_pair.left = 12
