#$ test: parse-pass
#$ rules: FFI-10

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    static visible_count: i32
    static mut changing_count: i32

fn main():
    pass
