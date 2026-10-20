#$ test: run-pass
#$ command: lint
#$ rules: LNT-6, FFI-15
#$ stdout: 5
#$ stdout: 3
## `[LNT-6]` — `L5002`: a conversion that copies at the FFI boundary, here `to_cstring`, whose result
## is passed to C. A conversion whose result stays in Ember is no lint.

import std.ffi

unsafe extern "C":
    @ffi(param(text, borrowed, nul_terminated))
    safe fn strlen(text: cstr) -> usize

fn main():
    value = "hello".to_cstring().unwrap()  #$ warning[L5002]: `to_cstring` copies its argument for a foreign call
    println(strlen(value.as_cstr()))
    kept = "abc".to_cstring().unwrap()
    println(kept.as_cstr().to_str().unwrap().len())
