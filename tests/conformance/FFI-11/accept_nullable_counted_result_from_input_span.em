#$ test: run-pass
#$ rules: FFI-10, FFI-11, LT-1a
#$ profiles: debug, release, shipping
#$ stdout: none
#$ 2
#$ 10

unsafe extern "C":
    @ffi(param(input, borrowed, count(n)), result(borrowed, count(n), nullable, from(input)), link_name="maybe_counted")
    safe fn maybe(input: Span[i32], n: usize, present: bool) -> Option[Span[i32]]

pub extern "C" fn maybe_counted(input: *i32, n: usize, present: bool) -> *i32:
    if present:
        return input
    return null[*i32]()

fn main():
    values: Array[i32] = Array[i32]()
    values.push(10)
    values.push(11)
    match maybe(values.as_span(), false):
        None:
            println("none")
        Some(result):
            println(result[0])
    match maybe(values.as_span(), true):
        None:
            println("none")
        Some(result):
            println(result.len())
            println(result[0])
