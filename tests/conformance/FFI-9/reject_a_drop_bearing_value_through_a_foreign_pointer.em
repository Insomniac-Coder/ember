#$ test: compile-fail
#$ rules: FFI-9, FFI-31b
#$ profiles: debug
# `[FFI-9]`, `[FFI-31b]` — a foreign function pointer is a boundary C may call
# through, and C cannot run an Ember `drop`: a value with drop glue may not
# cross it, as a parameter or a result, in any convention.

struct Packet:
    x: i32

    fn drop(mut self):
        pass

fn main():
    callback: extern "C" fn(Packet) -> i32 #$ error[E5015]: a value with drop glue may not cross an `extern` function pointer's boundary
    maker: extern "system" fn() -> Packet #$ error[E5015]: a value with drop glue may not cross an `extern` function pointer's boundary
