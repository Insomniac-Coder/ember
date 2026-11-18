#$ test: run-pass
#$ rules: TXT-3
#$ stdout: 3 true
#$ 3 3 ok
#$ the text has a NUL byte at index 1
## `[TXT-3]` - text is a pointer and a length and may hold `\0`; a C boundary uses `CString`, made from
## text with `to_cstring()`, which refuses an interior NUL (`NulError`, `[FFI-15]`).

import std.ffi

fn main():
    s = "a\0b"
    println(s.len(), s.as_bytes()[1] == 0)
    match "abc".to_cstring():
        Ok(_c):
            println("abc".len(), s.len(), "ok")
        Err(e):
            println(e)
    match s.to_cstring():
        Ok(_c):
            println("no")
        Err(e):
            println(e)
