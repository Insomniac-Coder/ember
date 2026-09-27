#$ test: run-pass
#$ rules: FFI-25, FFI-26
#$ profiles: debug
#$ assert-c: contains("int32_t default_abort(")
#$ assert-c: contains("int32_t named_abort(")
#$ stdout: 42
#$ 43

@export(on_panic=abort)
pub fn default_abort(value: i32) -> i32:
    return value + 1

@export("named_abort", on_panic=abort)
pub fn source_named(value: i32) -> i32:
    return value + 2

fn main():
    println(default_abort(41))
    println(source_named(41))
