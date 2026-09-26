#$ test: run-pass
#$ rules: FFI-10, FFI-11, LT-1a
#$ profiles: debug, release, shipping
#$ stdout: none
#$ 17

unsafe extern "C":
    @ffi(param(input, borrowed, fixed(1), exclusive), result(borrowed, one, nullable, exclusive, from(input)), link_name="maybe_echo_mut_number")
    safe fn maybe_echo(input: ref mut [i32; 1], present: bool) -> Option[ref mut i32]

pub extern "C" fn maybe_echo_mut_number(input: *mut i32, present: bool) -> *mut i32:
    if present:
        return input
    return null[*mut i32]()

fn main():
    value: [i32; 1] = [17]
    match maybe_echo(ref mut value, false):
        None => println("none")
        Some(chosen) => println(chosen)
    match maybe_echo(ref mut value, true):
        None => println("none")
        Some(chosen) => println(chosen)
