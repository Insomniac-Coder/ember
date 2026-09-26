#$ test: compile-fail
#$ rules: FFI-9, FFI-5
#$ profiles: debug
#$ error[E5050]: an `extern "C" fn` pointer needs FFI-safe parameters and result

fn main():
    callback: extern "C" fn(str) -> i32
