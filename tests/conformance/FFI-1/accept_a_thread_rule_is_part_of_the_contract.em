#$ test: run-pass
#$ rules: FFI-1, FFI-11
#$ profiles: debug, development, release
#$ stdout: 1 2
# `[FFI-1]`, `[FFI-11]` — a foreign declaration's contract carries its thread rule:
# `@ffi(threads(main))` says it may be called on the main thread only, and the debug profile
# checks each call (the run-fail case beside this one calls it from another thread). `ember tcb`
# lists the rule, and an unknown one where the contract states none.

import c "./ui.c" as raw

unsafe extern "C":
    @ffi(threads(main), effects=[], link_name="ui_frame")
    safe fn frame() -> i32

fn main():
    first = frame()
    println(first, frame())
