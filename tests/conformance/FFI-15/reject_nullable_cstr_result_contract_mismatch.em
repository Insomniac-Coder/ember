#$ test: compile-fail
#$ rules: FFI-10, FFI-11, FFI-15
#$ profiles: debug
#$ error[E5002]: `@ffi` nullable NUL-terminated result contract needs `Option[cstr]`

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(result(borrowed, nul_terminated, nullable, from(static)))
    safe fn wrong() -> cstr

fn main():
    pass
