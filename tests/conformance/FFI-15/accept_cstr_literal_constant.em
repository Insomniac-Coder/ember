#$ test: run-pass
#$ rules: FFI-15, FFI-11
#$ profiles: debug, release, shipping
#$ stdout: 5

const word: cstr = c"Ember"

unsafe extern "C":
    @ffi(param(text, borrowed, nul_terminated))
    safe fn strlen(text: cstr) -> usize

fn main():
    println(strlen(word))
