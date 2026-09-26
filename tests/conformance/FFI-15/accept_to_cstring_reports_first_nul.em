#$ test: run-pass
#$ rules: FFI-15, TXT-3, TXT-5
#$ profiles: debug, release, shipping
#$ stdout: 1

import std.ffi

fn main():
    match "a\0b\0".to_cstring():
        Ok(_) => println("unexpected")
        Err(problem) => println(problem.index)
