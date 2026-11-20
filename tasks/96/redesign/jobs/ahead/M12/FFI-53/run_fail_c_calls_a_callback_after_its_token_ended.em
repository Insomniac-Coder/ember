# Clause FFI-53.2: "Each call through the trampoline checks, in every profile, that the slot is inside
# the table, then the generation and the function's type in one compare"; "a call after the
# callback's token ended": "C called the callback registered on line 16 after its token ended; it
# was unregistered"
#$ test: run-fail
#$ rules: FFI-53, FFI-23
#$ profiles: debug, development, release
#$ stdout: 2
#$ panics: C called the callback registered on line 19 after its token ended; it was unregistered
# `kept.c`'s `unkeep` forgets nothing, so C calls the callback after its token ended: the stop names
# the registering line.

from std.ffi import c_int

# SAFETY: the test's own C, which runs nothing before `main`; its `unkeep` is wrong on purpose
unsafe import c "./kept.c" with (overlay = "overlays/kept.em") as k

fn main():
    token = k.keep(owns fn(v: c_int) -> c_int => v + 1)
    println(k.fire(1))
    mem.drop(owns token)
    println(k.fire(2))
