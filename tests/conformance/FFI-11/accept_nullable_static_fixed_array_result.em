#$ test: run-pass
#$ rules: FFI-10, FFI-11
#$ profiles: debug, release, shipping
#$ stdout: none
#$ 14

unsafe extern "C":
    @ffi(result(borrowed, fixed(2), nullable, from(static)), link_name="optional_pair")
    safe fn pair(present: bool) -> Option[ref [i32; 2]]

pub extern "C" fn optional_pair(present: bool) -> *i32:
    if not present:
        return null[*i32]()
    unsafe:
        data: *mut i32 = alloc[i32](2)
        write(data, 0, 14)
        write(data, 1, 15)
        return data as *i32

fn main():
    match pair(false):
        None => println("none")
        Some(value) => println(value[0])
    match pair(true):
        None => println("none")
        Some(value) => println(value[0])
