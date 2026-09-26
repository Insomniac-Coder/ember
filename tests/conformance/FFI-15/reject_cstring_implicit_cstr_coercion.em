#$ test: compile-fail
#$ rules: FFI-15, TYP-5
#$ profiles: debug
#$ error[E2020]: expected `cstr`, found `std.ffi.CString`

import std.ffi

unsafe extern "C":
    @ffi(param(text, borrowed, nul_terminated))
    safe fn strlen(text: cstr) -> usize

fn main():
    value = "hello".to_cstring().unwrap()
    println(strlen(value))
