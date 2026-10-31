#$ test: compile-fail
#$ rules: FFI-10, FFI-8
#$ profiles: debug
#$ error[E5015]: `Handle` is incomplete and cannot cross the C boundary by value

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    type Handle
    @safety("callers pass what the C function `consume` requires")
    unsafe fn consume(value: Handle)

fn main():
    pass
