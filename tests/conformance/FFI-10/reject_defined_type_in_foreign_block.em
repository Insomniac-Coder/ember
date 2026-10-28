#$ test: compile-fail
#$ rules: FFI-10
#$ profiles: debug
#$ error[E0100]: a foreign opaque type must be declared as `type Name`

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    type Handle = i32

fn main():
    pass
