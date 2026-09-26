#$ test: compile-fail
#$ rules: FFI-9, FFI-5
#$ profiles: debug
#$ error[E5050]: a native `fn` callable parameter has no foreign representation

pub extern "C" fn apply(callback: fn(i32) -> i32, value: i32) -> i32:
    return callback(value)
