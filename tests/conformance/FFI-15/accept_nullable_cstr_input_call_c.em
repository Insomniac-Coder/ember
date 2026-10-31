#$ test: run-pass
#$ rules: FFI-10, FFI-11, FFI-15, TYP-13
#$ profiles: debug, development, release
#$ stdout: 0
#$ stdout: 5

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(param(text, borrowed, nul_terminated, nullable), link_name="length_if_present")
    safe fn length(text: Option[CStrView]) -> usize

pub extern "C" fn length_if_present(text: Option[CStrView]) -> usize:
    match text:
        None:
            return 0
        Some(value):
            return strlen(value)

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(param(text, borrowed, nul_terminated))
    safe fn strlen(text: CStrView) -> usize

fn main():
    println(length(None))
    println(length(Some(c"hello")))
