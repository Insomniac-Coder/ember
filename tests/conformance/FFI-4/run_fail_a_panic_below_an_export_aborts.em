#$ test: run-fail
#$ rules: FFI-4, FFI-25
#$ panics: panic inside an exported function
#$ profiles: debug, development, release
# `[FFI-4]` — no Ember panic crosses into foreign code: a panic in an exported function, or
# anywhere below it, aborts the process (`[FFI-25]`), instead of unwinding into the caller.

@export("host_panic")
pub fn blow_up() -> i32:
    return deep_failure()

fn deep_failure() -> i32:
    panic("panic inside an exported function")

fn main():
    println(blow_up())
