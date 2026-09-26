#$ test: run-pass
#$ rules: FFI-10, FFI-2
#$ profiles: debug, release, shipping
#$ stdout: 42

unsafe extern "C":
    fn abs(x: i32) -> i32

fn main():
    unsafe:
        # C's abs has a scalar ABI; this declaration has not asserted safe-to-call.
        println(abs(-42))
