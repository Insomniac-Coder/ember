#$ test: run-fail
#$ rules: FFI-10, FFI-11
#$ profiles: debug
#$ panics: FFI span lengths differ

unsafe extern "C":
    @ffi(param(left, borrowed, count(n), nullable), param(right, borrowed, count(n)), link_name="count_optional_and_present")
    safe fn compare(left: Option[Span[u8]], right: Span[u8], n: usize) -> usize

pub extern "C" fn count_optional_and_present(left: *u8, right: *u8, n: usize) -> usize:
    return n

fn main():
    right: Array[u8] = Array[u8]()
    right.push(1)
    compare(None, right.as_span())
