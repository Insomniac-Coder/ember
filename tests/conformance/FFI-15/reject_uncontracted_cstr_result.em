#$ test: compile-fail
#$ rules: FFI-10, FFI-11, FFI-15
#$ profiles: debug
#$ error[E5002]: `safe fn` needs an `@ffi` contract for its `CStrView` result

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    safe fn get_name() -> CStrView

fn main():
    pass
