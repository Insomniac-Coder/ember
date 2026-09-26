#$ test: run-pass
#$ rules: FFI-10, FFI-11, MOD-2, LT-1a
#$ profiles: debug, release, shipping
#$ stdout: 42

from modules.ffi_static_result import static_result

fn main():
    input: i32 = 2
    answer = static_result(ref input)
    input = 3
    println(answer)
