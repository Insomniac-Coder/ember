#$ test: run-pass
#$ rules: FFI-10, FFI-11, LT-1a
#$ profiles: debug, release, shipping
#$ stdout: 13

unsafe extern "C":
    @ffi(param(input, borrowed, fixed(1), exclusive), result(borrowed, one, exclusive, from(input)), link_name="echo_mut_number")
    safe fn echo(input: ref mut [i32; 1]) -> ref mut i32

pub extern "C" fn echo_mut_number(input: *mut i32) -> *mut i32:
    return input

fn main():
    value: [i32; 1] = [13]
    chosen: ref mut i32 = echo(ref mut value)
    println(chosen)
