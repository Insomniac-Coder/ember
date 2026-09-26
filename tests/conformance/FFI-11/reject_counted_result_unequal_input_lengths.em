#$ test: run-fail
#$ rules: FFI-10, FFI-11
#$ profiles: debug
#$ panics: FFI span lengths differ

unsafe extern "C":
    @ffi(param(left, borrowed, count(n)), param(right, borrowed, count(n)), result(borrowed, count(n), from(left)), link_name="echo_shared_count")
    safe fn echo(left: Span[u8], right: Span[u8], n: usize) -> Span[u8]

pub extern "C" fn echo_shared_count(left: *u8, right: *u8, n: usize) -> *u8:
    return left

fn main():
    left: Array[u8] = Array[u8]()
    left.push(1)
    left.push(2)
    right: Array[u8] = Array[u8]()
    right.push(3)
    echo(left.as_span(), right.as_span())
