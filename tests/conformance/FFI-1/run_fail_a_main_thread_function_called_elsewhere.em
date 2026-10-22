#$ test: run-fail
#$ rules: FFI-1, FFI-11
#$ profiles: debug
#$ panics: `frame` whose contract says `threads(main)` is called on thread
# `[FFI-1]` — in the debug profile, a call of a foreign function contracted `threads(main)` from
# another thread panics before C runs, naming the function and both threads.

import std.thread
import c "./ui.c" as raw

unsafe extern "C":
    @ffi(threads(main), effects=[], link_name="ui_frame")
    safe fn frame() -> i32

fn main():
    worker = thread.spawn(owned fn() => frame())
    println(worker.join())
