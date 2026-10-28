#$ test: run-pass
#$ rules: FFI-10, FFI-11
#$ profiles: debug, development, release
#$ stdout: 4
#$ 5

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(result(borrowed, fixed(2), from(static)), link_name="process_pair")
    safe fn pair() -> ref [i32; 2]

pub extern "C" fn process_pair() -> *i32:
    unsafe:  # SAFETY(ffi): each foreign call matches its C declaration
        data: *mut i32 = alloc[i32](2)
        write(data, 0, 4)
        write(data, 1, 5)
        return data as *i32

fn main():
    value: ref [i32; 2] = pair()
    println(value[0])
    println(value[1])
