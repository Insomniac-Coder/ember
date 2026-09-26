#$ test: compile-fail
#$ rules: FFI-10, FFI-11, FFI-15
#$ profiles: debug
#$ error[E5002]: `safe fn` needs an `@ffi` contract for its `cstr` result

unsafe extern "C":
    safe fn get_name() -> cstr

fn main():
    pass
