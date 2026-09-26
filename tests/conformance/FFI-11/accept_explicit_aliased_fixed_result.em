#$ test: run-pass
#$ rules: FFI-10, FFI-11
#$ profiles: debug, release, shipping
#$ stdout: 31
#$ assert-c: contains("extern const int32_t* pair_c(")

unsafe extern "C":
    @ffi(result(borrowed, fixed(2), aliased, from(static)), link_name="pair_c")
    safe fn pair() -> ref [i32; 2]

pub extern "C" fn pair_c() -> *i32:
    unsafe:
        data: *mut i32 = alloc[i32](2)
        write(data, 0, 31)
        write(data, 1, 32)
        return data as *i32

fn main():
    println(pair()[0])
