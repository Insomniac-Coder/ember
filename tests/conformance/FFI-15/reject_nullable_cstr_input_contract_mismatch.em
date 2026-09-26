#$ test: compile-fail
#$ rules: FFI-10, FFI-11, FFI-15
#$ profiles: debug
#$ error[E5002]: `@ffi` contract for `text` needs a pointer carrier
#$ error[E5002]: `safe fn` needs an `@ffi` NUL-terminated contract for `text`

unsafe extern "C":
    @ffi(param(text, borrowed, nul_terminated, nullable))
    safe fn wrong(text: cstr) -> usize

fn main():
    pass
