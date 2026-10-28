#$ test: parse-pass
#$ rules: STA-1, FFI-10

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    static foreign_count: i32

fn main():
    unsafe:  # SAFETY(ffi): each foreign call matches its C declaration
        println(foreign_count)
