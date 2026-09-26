#$ test: parse-pass
#$ rules: FFI-10

unsafe extern "C":
    static visible_count: i32
    static mut changing_count: i32

fn main():
    pass
