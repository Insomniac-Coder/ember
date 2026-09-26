#$ test: run-pass
#$ rules: FFI-10, FFI-11
#$ profiles: debug, release, shipping
#$ stdout: 6

unsafe extern "C":
    @ffi(param(data, borrowed, fixed(3)), link_name="sum_three")
    safe fn sum(data: ref [i32; 3]) -> i32

pub extern "C" fn sum_three(data: *i32) -> i32:
    unsafe:
        return read(data, 0) + read(data, 1) + read(data, 2)

fn main():
    data: [i32; 3] = [1, 2, 3]
    println(sum(ref data))
