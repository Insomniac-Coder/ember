#$ test: run-pass
#$ rules: FFI-10, FFI-11
#$ profiles: debug, release, shipping
#$ stdout: none
#$ 42

unsafe extern "C":
    @ffi(result(borrowed, one, nullable, exclusive, from(static)), link_name="maybe_fresh_mut_number")
    safe fn maybe_fresh(present: bool) -> Option[ref mut i32]

pub extern "C" fn maybe_fresh_mut_number(present: bool) -> *mut i32:
    if not present:
        return null[*mut i32]()
    unsafe:
        number: *mut i32 = alloc[i32](1)
        write(number, 0, 42)
        return number

fn main():
    match maybe_fresh(false):
        None => println("none")
        Some(value) => println(value)
    match maybe_fresh(true):
        None => println("none")
        Some(value) => println(value)
