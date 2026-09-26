#$ test: run-pass
#$ rules: FFI-49, FFI-10
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ assert-c: contains("extern int32_t abs(")

unsafe extern "C":
    @ffi(link_name="abs")
    safe fn magnitude(x: i32) -> i32

fn main():
    println(magnitude(-42))
