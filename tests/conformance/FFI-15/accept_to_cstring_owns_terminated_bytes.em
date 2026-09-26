#$ test: run-pass
#$ rules: FFI-15, TXT-3, TXT-5
#$ profiles: debug, release, shipping
#$ stdout: 5
#$ stdout: 0

import std.ffi

unsafe extern "C":
    @ffi(param(text, borrowed, nul_terminated))
    safe fn strlen(text: cstr) -> usize

fn main():
    value = "hello".to_cstring().unwrap()
    println(strlen(value.as_cstr()))
    empty = "".to_cstring().unwrap()
    println(strlen(empty.as_cstr()))
