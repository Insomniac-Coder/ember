#$ test: run-pass
#$ rules: FFI-10, FFI-11, LT-1a
#$ profiles: debug, release, shipping
#$ stdout: 9

unsafe extern "C":
    @ffi(param(first, borrowed, one), param(second, borrowed, one),
         result(borrowed, one, from(first)), link_name="choose_first_c")
    safe fn choose_first(first: ref i32, second: ref i32) -> ref i32

pub extern "C" fn choose_first_c(first: *mut i32, second: *mut i32) -> *mut i32:
    unsafe:
        if read(second, 0) == 0:
            return first
    return first

fn main():
    first: i32 = 9
    second: i32 = 2
    chosen = choose_first(ref first, ref second)
    second = 11
    println(chosen)
