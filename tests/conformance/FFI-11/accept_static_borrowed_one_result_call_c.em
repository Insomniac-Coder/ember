#$ test: run-pass
#$ rules: FFI-10, FFI-11
#$ profiles: debug, release, shipping
#$ stdout: 42

unsafe extern "C":
    @ffi(result(borrowed, one, from(static)), link_name="process_number")
    safe fn static_number() -> ref i32

# The raw allocation lives for the process, satisfying from(static).
pub extern "C" fn process_number() -> *mut i32:
    unsafe:
        number: *mut i32 = alloc[i32](1)
        write(number, 0, 42)
        return number

fn main():
    println(static_number())
