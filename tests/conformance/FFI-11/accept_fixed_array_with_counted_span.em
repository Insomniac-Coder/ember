#$ test: run-pass
#$ rules: FFI-10, FFI-11
#$ profiles: debug, release, shipping
#$ stdout: 9

unsafe extern "C":
    @ffi(param(prefix, borrowed, fixed(2)), param(rest, borrowed, count(n)), link_name="sum_fixed_and_counted")
    safe fn mixed(prefix: ref [i32; 2], n: usize, rest: Span[i32]) -> i32

pub extern "C" fn sum_fixed_and_counted(prefix: *i32, n: usize, rest: *i32) -> i32:
    unsafe:
        return read(prefix, 0) + read(prefix, 1) + n as i32 + read(rest, 0)

fn main():
    prefix: [i32; 2] = [1, 2]
    rest: Array[i32] = Array[i32]()
    rest.push(5)
    println(mixed(ref prefix, rest.as_span()))
