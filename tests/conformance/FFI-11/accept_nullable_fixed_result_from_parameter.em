#$ test: run-pass
#$ rules: FFI-10, FFI-11, LT-1a
#$ profiles: debug, release, shipping
#$ stdout: none
#$ 20

unsafe extern "C":
    @ffi(param(input, borrowed, fixed(2)), result(borrowed, fixed(2), nullable, from(input)), link_name="maybe_pair")
    safe fn maybe(input: ref [i32; 2], present: bool) -> Option[ref [i32; 2]]

pub extern "C" fn maybe_pair(input: *i32, present: bool) -> *i32:
    if present:
        return input
    return null[*i32]()

fn main():
    data: [i32; 2] = [20, 21]
    match maybe(ref data, false):
        None => println("none")
        Some(value) => println(value[0])
    match maybe(ref data, true):
        None => println("none")
        Some(value) => println(value[0])
