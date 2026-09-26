#$ test: compile-fail
#$ rules: FFI-10, FFI-11, FFI-15
#$ profiles: debug
#$ error[E5002]: `safe fn` needs an `@ffi` nullable NUL-terminated contract for `text`

unsafe extern "C":
    safe fn undocumented(text: Option[cstr]) -> usize

fn main():
    pass
