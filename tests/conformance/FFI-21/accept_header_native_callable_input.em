#$ test: parse-pass
#$ rules: FFI-9, FFI-26

@export("native_callback_address")
pub fn native_callback_address(callback: *fn(i32) -> i32) -> i32:
    return 0
