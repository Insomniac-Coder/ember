# Clause FFI-53.3: "a `callback(once)` slot carries a used-up mark, taken with one atomic exchange
# before any of the callback's state is read"; "a `callback(once)` function called a second time":
# "C called the callback registered on line 29 a second time; it was registered `callback(once)`,
# so it may run only once"
#$ test: run-fail
#$ rules: FFI-53, FFI-21, CLO-6
#$ profiles: debug, development, release
#$ stdout: 5
#$ panics: C called the callback registered on line 18 a second time; it was registered `callback(once)`, so it may run only once
# `kept.c` keeps the `once` callback after running it, and `fire` calls it again.

from std.ffi import c_int

# SAFETY: the test's own C, which runs nothing before `main`; it keeps a `once` callback on purpose
unsafe import c "./kept.c" with (overlay = "overlays/kept.em") as k

fn main():
    _token = k.keep_once(owns fn(v: c_int) -> c_int => v + 4)
    println(k.fire(1))
    println(k.fire(2))
