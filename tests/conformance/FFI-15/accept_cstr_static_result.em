#$ test: run-pass
#$ rules: FFI-10, FFI-11, FFI-15, LT-1a
#$ profiles: debug, release, shipping
#$ stdout: 5

unsafe extern "C":
    @ffi(result(borrowed, nul_terminated, from(static)), link_name="get_word")
    safe fn word() -> cstr

pub extern "C" fn get_word() -> cstr:
    return c"Ember"

unsafe extern "C":
    @ffi(param(text, borrowed, nul_terminated))
    safe fn strlen(text: cstr) -> usize

fn main():
    println(strlen(word()))
