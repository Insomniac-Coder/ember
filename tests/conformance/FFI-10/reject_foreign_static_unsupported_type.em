#$ test: compile-fail
#$ rules: FFI-10, FFI-8
# `[FFI-10]` — `str` and `String` are Ember values with no C layout, so no C
# global holds one.

unsafe extern "C":
    static text: str    #$ error[E5050]: `str` has no foreign representation
    static mut owned_text: String    #$ error[E5050]: `String` has no foreign representation

fn main():
    pass
