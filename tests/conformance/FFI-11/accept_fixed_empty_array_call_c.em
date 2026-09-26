#$ test: run-pass
#$ rules: FFI-10, FFI-11
#$ profiles: debug, release, shipping
#$ stdout: 29

unsafe extern "C":
    @ffi(param(data, borrowed, fixed(0)), link_name="accept_empty")
    safe fn empty(data: ref [i32; 0]) -> i32

pub extern "C" fn accept_empty(data: *i32) -> i32:
    return 29

fn main():
    data: [i32; 0] = []
    println(empty(ref data))
