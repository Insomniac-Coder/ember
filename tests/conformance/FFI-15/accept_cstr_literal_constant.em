#$ test: run-pass
#$ rules: FFI-15, FFI-11
#$ profiles: debug, development, release
#$ stdout: 5

const word: CStrView = c"Ember"

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(param(text, borrowed, nul_terminated))
    safe fn strlen(text: CStrView) -> usize

fn main():
    println(strlen(word))
