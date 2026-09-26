#$ test: run-pass
#$ rules: FFI-10, FFI-11, LT-1a
#$ profiles: debug, release, shipping
#$ stdout: none
#$ 23

unsafe extern "C":
    @ffi(param(input, borrowed, fixed(2), exclusive), result(borrowed, fixed(2), nullable, exclusive, from(input)), link_name="maybe_mut_pair")
    safe fn maybe(input: ref mut [i32; 2], present: bool) -> Option[ref mut [i32; 2]]

pub extern "C" fn maybe_mut_pair(input: *mut i32, present: bool) -> *mut i32:
    if present:
        return input
    return null[*mut i32]()

fn main():
    data: [i32; 2] = [23, 24]
    match maybe(ref mut data, false):
        None => println("none")
        Some(value) => println(value[0])
    match maybe(ref mut data, true):
        None => println("none")
        Some(value) => println(value[0])
