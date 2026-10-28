#$ test: run-pass
#$ rules: FFI-15, TXT-5
#$ profiles: debug, development, release
#$ stdout: 6

import std.ffi

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(param(text, borrowed, nul_terminated))
    safe fn strlen(text: cstr) -> usize

fn main():
    source = String.from("Ember!")
    value = source.to_cstring().unwrap()
    println(strlen(value.as_cstr()))
