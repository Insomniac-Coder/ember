#$ test: run-pass
#$ rules: FFI-10, FFI-11
#$ profiles: debug, release, shipping
#$ stdout: 5
#$ 0

unsafe extern "C":
    @ffi(param(data, borrowed, fixed(2), nullable), link_name="sum_optional_fixed")
    safe fn sum(data: Option[ref [i32; 2]]) -> i32

pub extern "C" fn sum_optional_fixed(data: *i32) -> i32:
    if data.is_null():
        return 0
    unsafe:
        return read(data, 0) + read(data, 1)

fn main():
    data: [i32; 2] = [2, 3]
    println(sum(Some(ref data)))
    println(sum(None))
