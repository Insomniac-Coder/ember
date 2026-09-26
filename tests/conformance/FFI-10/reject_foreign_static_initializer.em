#$ test: parse-fail
#$ rules: FFI-10
#$ error[E0100]: a foreign static declaration has no initializer

unsafe extern "C":
    static visible_count: i32 = 5

fn main():
    pass
