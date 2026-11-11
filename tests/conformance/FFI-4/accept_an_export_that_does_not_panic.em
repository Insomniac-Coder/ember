#$ test: run-pass
#$ rules: FFI-4, FFI-25
#$ profiles: debug, development, release
#$ stdout: 42
# `[FFI-4]` — the only policy of an export is to abort on a panic (`[FFI-25]`); an export that
# does not panic returns its value as any function does.

@export(on_panic=abort)
pub fn plus_one(value: i32) -> i32:
    return value + 1

fn main():
    println(plus_one(41))
