#$ test: compile-fail
#$ rules: FFI-15, TXT-2, LT-1, BRW-4
#$ profiles: debug
#$ error[E3063]: this points into `value`

import std.ffi

fn dangling_text() -> StrView:
    value = "hello".to_cstring().unwrap()
    match value.as_cstr().to_str():
        Ok(text):
            return text
        Err(_):
            panic("impossible")

fn main():
    pass
