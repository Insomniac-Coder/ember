#$ test: run-pass
#$ rules: FFI-10, FFI-11, FFI-15, LT-1a
#$ profiles: debug, release, shipping
#$ stdout: none
#$ stdout: 4

unsafe extern "C":
    @ffi(param(text, borrowed, nul_terminated), result(borrowed, nul_terminated, nullable, from(text)), link_name="optional_echo_cstr")
    safe fn echo(text: cstr, present: bool) -> Option[cstr]

pub extern "C" fn optional_echo_cstr(text: cstr, present: bool) -> Option[cstr]:
    if present:
        return Some(text)
    return None

unsafe extern "C":
    @ffi(param(text, borrowed, nul_terminated))
    safe fn strlen(text: cstr) -> usize

fn main():
    match echo(c"safe", false):
        None => println("none")
        Some(text) => println(strlen(text))
    match echo(c"safe", true):
        None => println("unexpected")
        Some(text) => println(strlen(text))
