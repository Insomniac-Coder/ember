#$ test: compile-fail
#$ rules: FFI-10, FFI-9
#$ profiles: debug
# `[FFI-9]` names the calling conventions a declaration block may have; any
# other string names none.

# SAFETY(ffi): each declaration below matches its definition
unsafe extern "pascal":    #$ error[E0104]: `extern "pascal"` is not a calling convention
    safe fn abs(x: i32) -> i32

fn main():
    pass
