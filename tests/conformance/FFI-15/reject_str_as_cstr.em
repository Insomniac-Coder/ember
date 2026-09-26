#$ test: compile-fail
#$ rules: FFI-15
#$ profiles: debug
#$ error[E5020]: expected `cstr`, found `str`

unsafe extern "C":
    @ffi(param(text, borrowed, nul_terminated))
    safe fn strlen(text: cstr) -> usize

fn main():
    println(strlen("hello"))
