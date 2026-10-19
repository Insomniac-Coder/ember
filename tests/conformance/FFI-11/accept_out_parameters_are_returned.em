#$ test: run-pass
#$ rules: FFI-11, FFI-11b
#$ profiles: debug, release
#$ stdout: 7
#$ stdout: 3 4
#$ stdout: 1 2.5
# `[FFI-11]` — `param(p, out, one)` on `mut p: T`: C gets a `T*` to write, the
# parameter leaves the callable signature, and the call returns what C wrote.
# One output where C returns `void` is the result; several are a tuple in C's
# order; where C has a result of its own, it comes first (`[FFI-11b]`,
# ODR-137).

unsafe extern "C":
    @ffi(param(v, out, one), link_name="seven_c")
    safe fn seven(mut v: i32)

    @ffi(param(w, out, one), param(h, out, one), link_name="size_c")
    safe fn size(mut w: i32, id: i32, mut h: i32)

    @ffi(param(v, out, one), link_name="half_c")
    safe fn half(n: f64, mut v: f64) -> i32

pub extern "C" fn seven_c(v: *mut i32):
    unsafe:  # SAFETY(ffi): C's contract: `v` is valid
        write(v, 0, 7)

pub extern "C" fn size_c(w: *mut i32, id: i32, h: *mut i32):
    unsafe:  # SAFETY(ffi): C's contract: both are valid
        write(w, 0, id)
        write(h, 0, id + 1)

pub extern "C" fn half_c(n: f64, v: *mut f64) -> i32:
    unsafe:  # SAFETY(ffi): C's contract: `v` is valid
        write(v, 0, n / 2.0)
    return 1

fn main():
    println(seven())
    w, h = size(3)
    println(w, h)
    ok, v = half(5.0)
    println(ok, v)
