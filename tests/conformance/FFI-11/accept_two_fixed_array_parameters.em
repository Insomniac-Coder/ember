#$ test: run-pass
#$ rules: FFI-10, FFI-11
#$ profiles: debug, release, shipping
#$ stdout: 10

unsafe extern "C":
    @ffi(param(left, borrowed, fixed(2)), param(right, borrowed, fixed(2)), link_name="sum_pairs")
    safe fn sum(left: ref [i32; 2], right: ref [i32; 2]) -> i32

pub extern "C" fn sum_pairs(left: *i32, right: *i32) -> i32:
    unsafe:
        return read(left, 0) + read(left, 1) + read(right, 0) + read(right, 1)

fn main():
    left: [i32; 2] = [1, 2]
    right: [i32; 2] = [3, 4]
    println(sum(ref left, ref right))
