# Clause FFI-53.4: "then, for a `threads(creator)` or `threads(main)` callback, the calling thread";
# "a `threads(creator)` callback called on another thread": "C called the callback registered on
# line 16 on another thread, but its contract says `threads(creator)`"
#$ test: run-fail
#$ rules: FFI-53, FFI-22, FFI-33a
#$ profiles: debug, development, release
#$ stdout: 4
#$ panics: C called the callback registered on line 18 on another thread, but its contract says `threads(creator)`
# `fire` runs the callback on the thread that kept it; `fire_elsewhere` on a thread of its own, which
# the trampoline stops in every profile, before the callback's state is read.

from std.ffi import c_int

# SAFETY: the test's own C, which runs nothing before `main`; it calls a callback on the wrong thread on purpose
unsafe import c "./kept.c" with (overlay = "overlays/kept.em") as k

fn main():
    _token = k.keep_here(owns fn(v: c_int) -> c_int => v + 3)
    println(k.fire(1))
    println(k.fire_elsewhere(2))
