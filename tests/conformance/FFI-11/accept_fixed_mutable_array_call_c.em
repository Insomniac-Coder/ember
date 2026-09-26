#$ test: run-pass
#$ rules: FFI-10, FFI-11
#$ profiles: debug, release, shipping
#$ stdout: 17
#$ 17

unsafe extern "C":
    @ffi(param(data, borrowed, fixed(2), exclusive), link_name="fill_two")
    safe fn fill(data: ref mut [i32; 2]) -> i32

pub extern "C" fn fill_two(data: *mut i32) -> i32:
    unsafe:
        write(data, 0, 17)
        return read(data, 0)

fn main():
    data: [i32; 2] = [0, 0]
    println(fill(ref mut data))
    println(data[0])
