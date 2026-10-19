#$ test: run-pass
#$ rules: FFI-10, FFI-11
#$ profiles: debug, development, release
#$ stdout: 42

unsafe extern "C":
    @ffi(result(borrowed, one, exclusive, from(static)), link_name="fresh_mut_number")
    safe fn fresh() -> ref mut i32

pub extern "C" fn fresh_mut_number() -> *mut i32:
    number: *mut i32
    unsafe:  # SAFETY(ffi): each foreign call matches its C declaration
        number = alloc[i32](1)
        write(number, 0, 42)
    return number

fn main():
    number: ref mut i32 = fresh()
    println(number)
