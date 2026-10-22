#$ test: run-pass
#$ rules: FFI-49, FFI-10
#$ profiles: debug, development, release
#$ stdout: 42
#$ assert-c: contains("extern int32_t em_0x616273_9magnitude(")

unsafe extern "C":
    @ffi(link_name="abs")
    safe fn magnitude(x: i32) -> i32

fn main():
    println(magnitude(-42))
