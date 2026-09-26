#$ test: run-pass
#$ rules: FFI-10, FFI-11, FFI-15, TYP-13
#$ profiles: debug, release, shipping
#$ stdout: none
#$ stdout: 5

unsafe extern "C":
    @ffi(result(borrowed, nul_terminated, nullable, from(static)), link_name="choose_optional_cstr")
    safe fn choose(present: bool) -> Option[cstr]

pub extern "C" fn choose_optional_cstr(present: bool) -> Option[cstr]:
    if present:
        return Some(c"hello")
    return None

unsafe extern "C":
    @ffi(param(text, borrowed, nul_terminated))
    safe fn strlen(text: cstr) -> usize

fn main():
    match choose(false):
        None => println("none")
        Some(text) => println(strlen(text))
    match choose(true):
        None => println("unexpected")
        Some(text) => println(strlen(text))
