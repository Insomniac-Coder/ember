#$ test: run-fail
#$ rules: FFI-10, FFI-11
#$ profiles: debug
#$ panics: FFI count does not fit

unsafe extern "C":
    @ffi(param(data, borrowed, count(n)), link_name="probe_count_u8")
    safe fn counted(data: Span[u8], n: u8) -> i32

pub extern "C" fn probe_count_u8(data: *u8, n: u8) -> i32:
    unsafe:
        return read(data, 0) as i32 + n as i32

fn main():
    data: Array[u8] = Array[u8]()
    i: usize = 0
    while i < 256:
        data.push(1)
        i = i + 1
    counted(data.as_span())
