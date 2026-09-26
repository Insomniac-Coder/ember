#$ test: parse-pass
#$ rules: STA-1, FFI-10

unsafe extern "C":
    static foreign_count: i32

fn main():
    unsafe:
        println(foreign_count)
