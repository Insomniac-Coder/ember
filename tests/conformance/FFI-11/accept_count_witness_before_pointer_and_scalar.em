#$ test: run-pass
#$ rules: FFI-10, FFI-11
#$ profiles: debug, release, shipping
#$ stdout: 8

unsafe extern "C":
    @ffi(param(data, borrowed, count(n)), link_name="probe_order")
    safe fn counted(marker: i32, n: usize, data: Span[u8]) -> i32

pub extern "C" fn probe_order(marker: i32, n: usize, data: *u8) -> i32:
    unsafe:
        return marker + n as i32 + read(data, 0) as i32

fn main():
    data: Array[u8] = Array[u8]()
    data.push(1)
    data.push(2)
    println(counted(5, data.as_span()))
