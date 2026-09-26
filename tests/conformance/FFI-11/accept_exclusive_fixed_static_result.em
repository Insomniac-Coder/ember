#$ test: run-pass
#$ rules: FFI-10, FFI-11
#$ profiles: debug, release, shipping
#$ stdout: 12
#$ 9

unsafe extern "C":
    @ffi(result(borrowed, fixed(2), exclusive, from(static)), link_name="fresh_mut_pair")
    safe fn pair() -> ref mut [i32; 2]

pub extern "C" fn fresh_mut_pair() -> *mut i32:
    unsafe:
        data: *mut i32 = alloc[i32](2)
        write(data, 0, 8)
        write(data, 1, 9)
        return data

fn main():
    value: ref mut [i32; 2] = pair()
    value[0] = 12
    println(value[0])
    println(value[1])
