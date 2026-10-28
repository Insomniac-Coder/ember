#$ test: run-pass
#$ rules: FFI-10, FFI-11, TYP-13
#$ profiles: debug, development, release
#$ stdout: some
#$ 0

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(param(input, borrowed, count(n)), result(borrowed, count(n), nullable, exclusive, from(static)), link_name="empty_mut_some")
    safe fn maybe(input: Span[i32], n: usize) -> Option[MutSpan[i32]]

pub extern "C" fn empty_mut_some(input: *i32, n: usize) -> *mut i32:
    unsafe:  # SAFETY(ffi): each foreign call matches its C declaration
        return alloc[i32](1)

fn main():
    values: Array[i32] = Array[i32]()
    match maybe(values.as_span()):
        None:
            println("none")
        Some(result):
            println("some")
            println(result.len())
