#$ test: parse-pass
#$ rules: FFI-25, FFI-26, PAN-1

@export("host_panic")
pub fn blow_up() -> i32:
    return deep_failure()

fn deep_failure() -> i32:
    panic("panic inside an exported function")

unsafe extern "C":
    safe fn call_exported_panic() -> i32

fn main():
    println("before host call")
    println(call_exported_panic())
    println("after host call")
