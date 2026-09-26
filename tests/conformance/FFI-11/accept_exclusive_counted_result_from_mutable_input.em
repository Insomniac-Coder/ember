#$ test: run-pass
#$ rules: FFI-10, FFI-11, LT-1a
#$ profiles: debug, release, shipping
#$ stdout: 2
#$ 29
#$ 29

unsafe extern "C":
    @ffi(param(input, borrowed, count(n), exclusive), result(borrowed, count(n), exclusive, from(input)), link_name="echo_mut_span")
    safe fn echo(input: MutSpan[i32], n: usize) -> MutSpan[i32]

pub extern "C" fn echo_mut_span(input: *mut i32, n: usize) -> *mut i32:
    return input

fn main():
    values: Array[i32] = Array[i32]()
    values.push(7)
    values.push(8)
    result: MutSpan[i32] = echo(values.as_mut_span())
    result[0] = 29
    println(result.len())
    println(result[0])
    println(values[0])
