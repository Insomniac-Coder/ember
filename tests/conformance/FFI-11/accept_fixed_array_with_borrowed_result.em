#$ test: run-pass
#$ rules: FFI-10, FFI-11, LT-1a
#$ profiles: debug, release, shipping
#$ stdout: 12

unsafe extern "C":
    @ffi(param(input, borrowed, fixed(2)), result(borrowed, one, from(input)), link_name="first_fixed")
    safe fn first(input: ref [i32; 2]) -> ref i32

pub extern "C" fn first_fixed(input: *i32) -> *i32:
    return input

fn main():
    values: [i32; 2] = [12, 13]
    chosen: ref i32 = first(ref values)
    println(chosen)
