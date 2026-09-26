#$ test: run-pass
#$ rules: FFI-10, FFI-11, TYP-13
#$ profiles: debug, release, shipping
#$ stdout: some
#$ 0

unsafe extern "C":
    @ffi(param(input, borrowed, count(n)), result(borrowed, count(n), nullable, from(static)), link_name="empty_some")
    safe fn maybe(input: Span[i32], n: usize) -> Option[Span[i32]]

pub extern "C" fn empty_some(input: *i32, n: usize) -> *i32:
    unsafe:
        storage: *mut i32 = alloc[i32](1)
        return storage as *i32

fn main():
    values: Array[i32] = Array[i32]()
    match maybe(values.as_span()):
        None:
            println("none")
        Some(result):
            println("some")
            println(result.len())
