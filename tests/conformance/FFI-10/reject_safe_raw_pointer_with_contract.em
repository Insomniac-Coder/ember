#$ test: compile-fail
#$ rules: FFI-10, FFI-11
#$ profiles: debug
#$ error[E5002]: `safe fn` cannot expose raw pointer parameter `p`

unsafe extern "C":
    @ffi(param(p, borrowed, one, exclusive))
    safe fn inspect(p: *u8) -> i32

fn main():
    pass
