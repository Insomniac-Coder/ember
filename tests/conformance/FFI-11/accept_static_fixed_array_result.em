#$ test: run-pass
#$ rules: FFI-10, FFI-11
#$ profiles: debug, release, shipping
#$ stdout: 4
#$ 5

unsafe extern "C":
    @ffi(result(borrowed, fixed(2), from(static)), link_name="process_pair")
    safe fn pair() -> ref [i32; 2]

pub extern "C" fn process_pair() -> *i32:
    unsafe:
        data: *mut i32 = alloc[i32](2)
        write(data, 0, 4)
        write(data, 1, 5)
        return data as *i32

fn main():
    value: ref [i32; 2] = pair()
    println(value[0])
    println(value[1])
