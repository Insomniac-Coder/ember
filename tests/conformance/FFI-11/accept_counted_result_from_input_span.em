#$ test: run-pass
#$ rules: FFI-10, FFI-11, LT-1a
#$ profiles: debug, release, shipping
#$ stdout: 2
#$ 7

unsafe extern "C":
    @ffi(param(input, borrowed, count(n)), result(borrowed, count(n), from(input)), link_name="echo_counted")
    safe fn echo(input: Span[i32], n: usize) -> Span[i32]

pub extern "C" fn echo_counted(input: *i32, n: usize) -> *i32:
    return input

fn main():
    values: Array[i32] = Array[i32]()
    values.push(7)
    values.push(8)
    result: Span[i32] = echo(values.as_span())
    println(result.len())
    println(result[0])
