#$ test: run-pass
#$ rules: FFI-10, FFI-11, FFI-15, LT-1a
#$ profiles: debug, development, release
#$ stdout: none
#$ stdout: 6

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(param(text, borrowed, nul_terminated, nullable), result(borrowed, nul_terminated, nullable, from(text)), link_name="optional_cstr_identity")
    safe fn identity(text: Option[CStrView]) -> Option[CStrView]

pub extern "C" fn optional_cstr_identity(text: Option[CStrView]) -> Option[CStrView]:
    return text

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(param(text, borrowed, nul_terminated))
    safe fn strlen(text: CStrView) -> usize

fn main():
    match identity(None):
        None => println("none")
        Some(text) => println(strlen(text))
    match identity(Some(c"Ember!")):
        None => println("unexpected")
        Some(text) => println(strlen(text))
