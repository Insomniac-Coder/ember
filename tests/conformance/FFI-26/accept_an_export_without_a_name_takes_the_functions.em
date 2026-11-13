#$ test: run-pass
#$ rules: FFI-26
#$ assert-c: contains("int32_t door_score(")
#$ assert-c: contains("int32_t door_threads(")
#$ stdout: 7 8
# `[FFI-26]` — without a name, the export's C symbol is the function's own name, with or without the
# attribute's other arguments.

# SAFETY: the host defines no other symbol named `door_score`
@unsafe(export())
fn door_score() -> i32:
    return 7

# SAFETY: the host defines no other symbol named `door_threads`
@unsafe(export(threads=any))
fn door_threads() -> i32:
    return 8

fn main():
    println(door_score(), door_threads())
