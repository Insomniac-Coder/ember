#$ test: run-pass
#$ rules: FFI-10, FFI-11
#$ profiles: debug, development, release
#$ stdout: none
#$ 42

unsafe extern "C":
    @ffi(result(borrowed, one, nullable, from(static)), link_name="process_optional")
    safe fn optional_number(present: bool) -> Option[ref i32]

# The allocated Some value lives for the process; None is the C null pointer.
pub extern "C" fn process_optional(present: bool) -> *mut i32:
    if not present:
        unsafe:  # SAFETY(ffi): each foreign call matches its C declaration
            return 0 as *mut i32
    number: *mut i32
    unsafe:  # SAFETY(ffi): each foreign call matches its C declaration
        number = alloc[i32](1)
        write(number, 0, 42)
    return number

fn main():
    match optional_number(false):
        None => println("none")
        Some(value) => println(value)
    match optional_number(true):
        None => println("none")
        Some(value) => println(value)
