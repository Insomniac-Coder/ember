#$ test: run-pass
#$ rules: FFI-10, FFI-11
#$ profiles: debug, release, shipping
#$ stdout: 1
#$ 23
#$ 0

unsafe extern "C":
    @ffi(param(data, borrowed, fixed(2), nullable, exclusive), link_name="fill_optional_fixed")
    safe fn fill(data: Option[ref mut [i32; 2]]) -> i32

pub extern "C" fn fill_optional_fixed(data: *mut i32) -> i32:
    if data.is_null():
        return 0
    unsafe:
        write(data, 0, 23)
    return 1

fn main():
    data: [i32; 2] = [0, 0]
    println(fill(Some(ref mut data)))
    println(data[0])
    println(fill(None))
