#$ test: run-pass
#$ rules: FFI-22, FFI-25
#$ profiles: debug, development, release
#$ assert-c: contains("ember_rt_thread_attach();")
#$ assert-c: contains("int32_t attached_score(")
#$ stdout: 42
# `[FFI-22]` — every exported function first attaches the calling thread to the runtime
# (`ember_rt_thread_attach()`, a thread-local check after the first time).

@export(on_panic=abort, "attached_score")
pub fn score(value: i32) -> i32:
    return value + 1

fn main():
    println(score(41))
