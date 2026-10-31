#$ test: compile-fail
#$ rules: FFI-49
#$ profiles: debug
# `[FFI-49]` — a `link_name` is a symbol a linker can take: no space, quote, backslash or control
# character, which no object format carries.

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(link_name="two words")  #$ error[E0104]: `two words` is not a symbol a linker takes
    @safety("callers pass what the C function `magnitude` requires")
    unsafe fn magnitude(x: i32) -> i32

fn main():
    pass
