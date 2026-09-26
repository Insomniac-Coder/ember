#$ test: run-pass
#$ rules: FFI-10, FFI-11, LT-1a
#$ profiles: debug, release, shipping
#$ stdout: none
#$ 2
#$ 37

unsafe extern "C":
    @ffi(param(input, borrowed, count(n), exclusive), result(borrowed, count(n), nullable, exclusive, from(input)), link_name="maybe_mut_span")
    safe fn maybe(input: MutSpan[i32], n: usize, present: bool) -> Option[MutSpan[i32]]

pub extern "C" fn maybe_mut_span(input: *mut i32, n: usize, present: bool) -> *mut i32:
    if present:
        return input
    return null[*mut i32]()

fn main():
    values: Array[i32] = Array[i32]()
    values.push(3)
    values.push(4)
    match maybe(values.as_mut_span(), false):
        None:
            println("none")
        Some(result):
            println(result.len())
    match maybe(values.as_mut_span(), true):
        None:
            println("none")
        Some(result):
            result[0] = 37
            println(result.len())
            println(result[0])
