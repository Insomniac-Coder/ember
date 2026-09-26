#$ test: run-fail
#$ rules: FFI-10, FFI-11
#$ profiles: debug
#$ panics: FFI span lengths differ

unsafe extern "C":
    @ffi(param(left, borrowed, count(n)), param(right, borrowed, count(n)), link_name="probe_bytes")
    safe fn counted_pair(left: Span[u8], right: Span[u8], n: usize) -> i32

pub extern "C" fn probe_bytes(left: *u8, right: *u8, n: usize) -> i32:
    unsafe:
        return read(left, 0) as i32 + read(right, 0) as i32 + n as i32

fn main():
    left: Array[u8] = Array[u8]()
    left.push(1)
    left.push(2)
    right: Array[u8] = Array[u8]()
    right.push(1)
    counted_pair(left.as_span(), right.as_span())
