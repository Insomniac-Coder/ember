#$ test: run-pass
#$ rules: FFI-10, FFI-11, FFI-15, TYP-13
#$ profiles: debug, release, shipping
#$ stdout: 0
#$ stdout: 5

unsafe extern "C":
    @ffi(param(text, borrowed, nul_terminated, nullable), link_name="length_if_present")
    safe fn length(text: Option[cstr]) -> usize

pub extern "C" fn length_if_present(text: Option[cstr]) -> usize:
    match text:
        None:
            return 0
        Some(value):
            unsafe:
                return strlen(value)

unsafe extern "C":
    @ffi(param(text, borrowed, nul_terminated))
    safe fn strlen(text: cstr) -> usize

fn main():
    println(length(None))
    println(length(Some(c"hello")))
