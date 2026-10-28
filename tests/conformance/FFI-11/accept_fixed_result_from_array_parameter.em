#$ test: run-pass
#$ rules: FFI-10, FFI-11, LT-1a
#$ profiles: debug, development, release
#$ stdout: 8

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(param(input, borrowed, fixed(2)), result(borrowed, fixed(2), from(input)), link_name="echo_pair")
    safe fn echo(input: ref [i32; 2]) -> ref [i32; 2]

pub extern "C" fn echo_pair(input: *i32) -> *i32:
    return input

fn main():
    data: [i32; 2] = [8, 9]
    result: ref [i32; 2] = echo(ref data)
    println(result[0])
