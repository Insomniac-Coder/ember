#$ test: compile-fail
#$ rules: FFI-10
#$ profiles: debug
#$ error[E0100]: a foreign opaque type must be declared as `type Name`

unsafe extern "C":
    type Handle = i32

fn main():
    pass
