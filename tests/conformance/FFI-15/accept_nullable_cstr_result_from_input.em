#$ test: run-pass
#$ rules: FFI-10, FFI-11, FFI-15, LT-1a
#$ profiles: debug, development, release
#$ stdout: none
#$ stdout: 4

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(param(text, borrowed, nul_terminated), result(borrowed, nul_terminated, nullable, from(text)), link_name="optional_echo_cstr")
    safe fn echo(text: CStrView, present: bool) -> Option[CStrView]

pub extern "C" fn optional_echo_cstr(text: CStrView, present: bool) -> Option[CStrView]:
    if present:
        return Some(text)
    return None

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(param(text, borrowed, nul_terminated))
    safe fn strlen(text: CStrView) -> usize

fn main():
    match echo(c"safe", false):
        None => println("none")
        Some(text) => println(strlen(text))
    match echo(c"safe", true):
        None => println("unexpected")
        Some(text) => println(strlen(text))
