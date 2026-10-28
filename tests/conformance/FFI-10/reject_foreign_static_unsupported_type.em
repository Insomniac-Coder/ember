#$ test: compile-fail
#$ rules: FFI-10, FFI-8
# `[FFI-10]` — `str` and `String` are Ember values with no C layout, so no C
# global holds one.

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    static text: str    #$ error[E5015]: `str` has no foreign representation
    static mut owned_text: String    #$ error[E5015]: `String` has no foreign representation

fn main():
    pass
