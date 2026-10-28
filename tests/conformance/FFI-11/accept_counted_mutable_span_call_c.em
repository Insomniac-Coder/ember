#$ test: run-pass
#$ rules: FFI-10, FFI-11
#$ profiles: debug, development, release
#$ stdout: 1
#$ 7

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(param(data, borrowed, count(n), exclusive), link_name="fill_bytes")
    safe fn fill(data: MutSpan[u8], n: usize) -> i32

pub extern "C" fn fill_bytes(data: *mut u8, n: usize) -> i32:
    unsafe:  # SAFETY(ffi): each foreign call matches its C declaration
        write(data, 0, 7)
    return n as i32

fn main():
    values: Array[u8] = Array[u8]()
    values.push(0)
    println(fill(values.as_mut_span()))
    println(values[0])
