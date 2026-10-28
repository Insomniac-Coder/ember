#$ test: run-pass
#$ rules: FFI-10, FFI-11, LT-1a
#$ profiles: debug, development, release
#$ stdout: 42

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(param(input, borrowed, one), result(borrowed, one, from(static)), link_name="static_despite_input_c")
    safe fn static_despite_input(input: ref i32) -> ref i32

pub extern "C" fn static_despite_input_c(input: *mut i32) -> *mut i32:
    number: *mut i32
    unsafe:  # SAFETY(ffi): each foreign call matches its C declaration
        number = alloc[i32](1)
        write(number, 0, 41 + read(input, 0))
    return number

fn main():
    input: i32 = 1
    answer = static_despite_input(ref input)
    input = 2
    println(answer)
