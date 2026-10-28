#$ test: run-pass
#$ rules: FFI-10, FFI-11, FFI-15, LT-1a
#$ profiles: debug, development, release
#$ stdout: 6

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(param(text, borrowed, nul_terminated), result(borrowed, nul_terminated, from(text)), link_name="echo_cstr")
    safe fn echo(text: cstr) -> cstr

pub extern "C" fn echo_cstr(text: cstr) -> cstr:
    return text

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(param(text, borrowed, nul_terminated))
    safe fn strlen(text: cstr) -> usize

fn main():
    println(strlen(echo(c"Ember!")))
