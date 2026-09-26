#$ test: run-pass
#$ rules: FFI-10, FFI-11
#$ profiles: debug, release, shipping
#$ stdout: 3

unsafe extern "C":
    @ffi(param(data, borrowed, count(n)), link_name="probe_count_u8")
    safe fn counted(data: Span[u8], n: u8) -> i32

pub extern "C" fn probe_count_u8(data: *u8, n: u8) -> i32:
    unsafe:
        return read(data, 0) as i32 + n as i32

fn main():
    data: Array[u8] = Array[u8]()
    data.push(1)
    data.push(2)
    println(counted(data.as_span()))
