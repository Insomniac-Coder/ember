# Clause FFI-53.1: "C never holds the address of an Ember value for a callback: the user data it is
# given is a tagged number, the callback's slot in a table that never moves in the low 48 bits and
# the slot's generation in the top 16, which always has its bits 0, 7 and 15 set (bits 48, 55 and 63
# of the number"
#$ test: run-pass
#$ rules: FFI-53, FFI-21, FFI-23
#$ profiles: debug, development, release
#$ stdout: 1
#$ stdout: 6
# The user data C keeps has bits 48, 55 and 63 set (`kept.c`'s `tagged`), and the trampoline it
# calls runs the function, in every profile.

from std.ffi import c_int

# SAFETY: the test's own C, which runs nothing before `main`
unsafe import c "./kept.c" with (overlay = "overlays/kept.em") as k

fn main():
    _token = k.keep(owns fn(v: c_int) -> c_int => v * 2)
    println(k.tagged())
    println(k.fire(3))
