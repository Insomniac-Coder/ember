#$ test: parse-fail
#$ rules: FFI-10
#$ error[E0100]: a foreign static declaration has no initializer

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    static visible_count: i32 = 5

fn main():
    pass
