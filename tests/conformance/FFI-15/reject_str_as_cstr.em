#$ test: compile-fail
#$ rules: FFI-15
#$ profiles: debug
#$ error[E5020]: expected `CStrView`, found `StrView`

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(param(text, borrowed, nul_terminated))
    safe fn strlen(text: CStrView) -> usize

fn main():
    println(strlen("hello"))
