#$ test: compile-fail
#$ rules: FFI-11

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(result(borrowed, one, exclusive, aliased, from(static))) #$ error[E0104]: a contract is `exclusive` or `aliased`, not both
    safe fn contradictory() -> ref mut i32 #$ error[E5002]: `safe fn` needs an `@ffi` contract for reference result

fn main():
    pass
