#$ test: run-pass
#$ rules: FFI-10, FFI-11, LT-1a
#$ profiles: debug, release, shipping
#$ stdout: 18

unsafe extern "C":
    @ffi(param(input, borrowed, count(n)), result(borrowed, one, from(input)), link_name="first_counted")
    safe fn first(input: Span[i32], n: usize) -> ref i32

pub extern "C" fn first_counted(input: *i32, n: usize) -> *i32:
    return input

fn main():
    values: Array[i32] = Array[i32]()
    values.push(18)
    chosen: ref i32 = first(values.as_span())
    println(chosen)
