#$ test: run-pass
#$ rules: FFI-15, FFI-11, FFI-10
#$ profiles: debug, release, shipping
#$ stdout: 5
#$ 1

unsafe extern "C":
    @ffi(param(text, borrowed, nul_terminated))
    safe fn strlen(text: cstr) -> usize

fn main():
    println(strlen(c"hello"))
    println(strlen(c"\xFF"))
