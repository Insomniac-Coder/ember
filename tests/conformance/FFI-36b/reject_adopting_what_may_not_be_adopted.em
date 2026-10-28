#$ test: compile-fail
#$ rules: FFI-36b, FFI-36a
# `[FFI-36b]` — a foreign type whose contract says `adopt=false` keeps its
# objects the library's: adopting one is `E5052`.

from std.ffi import adopt

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(adopt=false, destroys(pool_free))
    type Pool
    fn pool_default() -> *mut Pool
    fn pool_free(pool: *mut Pool)

fn main():
    unsafe:  # SAFETY(ffi): the default pool lives as long as the library
        pool = adopt(pool_default())    #$ error[E5052]: a `Pool` may not be adopted: its declaration says `adopt=false`
