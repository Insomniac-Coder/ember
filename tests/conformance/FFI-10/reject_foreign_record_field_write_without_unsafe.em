#$ test: compile-fail
#$ rules: FFI-10, STA-1
#$ error[E5002]: writing a foreign static requires `unsafe`

@derive(Copy)
struct Pair:
    left: i32

unsafe extern "C":
    static mut foreign_pair: Pair

fn main():
    foreign_pair.left = 12
