#$ test: run-pass
#$ rules: FFI-10, FFI-11
#$ profiles: debug, release, shipping
#$ stdout: 1
#$ 2
#$ 3
#$ 41

unsafe extern "C":
    @ffi(param(data, borrowed, count(n), nullable, exclusive), link_name="inspect_optional_mut_span")
    safe fn inspect(data: Option[MutSpan[i32]], n: usize) -> i32

pub extern "C" fn inspect_optional_mut_span(data: *mut i32, n: usize) -> i32:
    if data.is_null():
        return 1
    if n == 0:
        return 2
    unsafe:
        write(data, 0, 41)
    return 3

fn main():
    empty: Array[i32] = Array[i32]()
    values: Array[i32] = Array[i32]()
    values.push(7)
    println(inspect(None))
    println(inspect(Some(empty.as_mut_span())))
    println(inspect(Some(values.as_mut_span())))
    println(values[0])
