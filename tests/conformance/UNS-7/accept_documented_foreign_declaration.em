#$ test: compile-pass
#$ rules: UNS-7, FFI-10
#$ profiles: debug

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @safety("The caller provides a valid C argument.")
    fn abs(x: i32) -> i32

fn main():
    pass
