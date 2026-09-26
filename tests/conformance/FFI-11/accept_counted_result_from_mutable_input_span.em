#$ test: run-pass
#$ rules: FFI-10, FFI-11, LT-1a
#$ profiles: debug, release, shipping
#$ stdout: 1
#$ 27

unsafe extern "C":
    @ffi(param(input, borrowed, count(n), exclusive), result(borrowed, count(n), from(input)), link_name="echo_mut_counted")
    safe fn echo(input: MutSpan[i32], n: usize) -> Span[i32]

pub extern "C" fn echo_mut_counted(input: *mut i32, n: usize) -> *i32:
    unsafe:
        return input as *i32

fn main():
    values: Array[i32] = Array[i32]()
    values.push(27)
    result: Span[i32] = echo(values.as_mut_span())
    println(result.len())
    println(result[0])
