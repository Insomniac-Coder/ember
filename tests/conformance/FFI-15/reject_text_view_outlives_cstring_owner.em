#$ test: compile-fail
#$ rules: FFI-15, TXT-2, LT-1, BRW-4
#$ profiles: debug
#$ error[E3060]: `value` does not live long enough

import std.ffi

fn dangling_text() -> str:
    value = "hello".to_cstring().unwrap()
    match value.as_cstr().to_str():
        Ok(text):
            return text
        Err(_):
            panic("impossible")

fn main():
    pass
