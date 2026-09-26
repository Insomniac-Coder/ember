#$ test: compile-fail
#$ rules: FFI-10
#$ profiles: debug
#$ error[E5002]: a foreign declaration block must be `unsafe extern`

extern "C":
    safe fn abs(x: i32) -> i32

fn main():
    pass
