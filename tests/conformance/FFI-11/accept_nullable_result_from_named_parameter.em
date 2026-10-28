#$ test: run-pass
#$ rules: FFI-10, FFI-11, LT-1a
#$ profiles: debug, development, release
#$ stdout: 9

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(param(first, borrowed, one), param(second, borrowed, one),
         result(borrowed, one, nullable, from(first)), link_name="optional_first_c")
    safe fn optional_first(first: ref i32, second: ref i32) -> Option[ref i32]

pub extern "C" fn optional_first_c(first: *mut i32, second: *mut i32) -> *mut i32:
    unsafe:  # SAFETY(ffi): each foreign call matches its C declaration
        if read(second, 0) == 0:
            return 0 as *mut i32
    return first

fn main():
    first: i32 = 9
    second: i32 = 2
    chosen = optional_first(ref first, ref second)
    second = 11
    match chosen:
        None => println("none")
        Some(value) => println(value)
