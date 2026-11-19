#$ test: compile-fail
#$ rules: FFI-15, LT-1, BRW-4
#$ profiles: debug
#$ error[E3063]: this points into `value`

import std.ffi

fn dangling() -> CStrView:
    value = "hello".to_cstring().unwrap()
    return value.as_cstr()

fn main():
    pass
