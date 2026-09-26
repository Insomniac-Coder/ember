#$ test: run-pass
#$ rules: FFI-10, FFI-11
#$ profiles: debug, release, shipping
#$ stdout: 1
#$ 2
#$ 3

unsafe extern "C":
    @ffi(param(data, borrowed, count(n), nullable), link_name="inspect_optional_span")
    safe fn inspect(data: Option[Span[i32]], n: usize) -> i32

pub extern "C" fn inspect_optional_span(data: *i32, n: usize) -> i32:
    if data.is_null():
        return 1
    if n == 0:
        return 2
    return 3

fn main():
    empty: Array[i32] = Array[i32]()
    values: Array[i32] = Array[i32]()
    values.push(7)
    println(inspect(None))
    println(inspect(Some(empty.as_span())))
    println(inspect(Some(values.as_span())))
