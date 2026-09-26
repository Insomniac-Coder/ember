#$ test: compile-fail
#$ rules: FFI-15, LT-1, BRW-4
#$ profiles: debug
#$ error[E3060]: `value` does not live long enough

import std.ffi

fn dangling() -> cstr:
    value = "hello".to_cstring().unwrap()
    return value.as_cstr()

fn main():
    pass
