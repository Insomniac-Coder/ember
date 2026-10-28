#$ test: run-pass
#$ rules: FFI-15, FFI-11, FFI-10
#$ profiles: debug, development, release
#$ stdout: 5
#$ 1

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(param(text, borrowed, nul_terminated))
    safe fn strlen(text: cstr) -> usize

fn main():
    println(strlen(c"hello"))
    println(strlen(c"\xFF"))
