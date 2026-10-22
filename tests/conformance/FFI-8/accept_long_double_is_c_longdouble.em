#$ test: run-pass
#$ rules: FFI-8, FFI-5
#$ profiles: debug, development, release
#$ stdout: 0.6667 0.3333
#$ stdout: 3 0.75
#$ stdout: true
# `[FFI-8]` — C's `long double` is `c_longdouble`: opaque bytes of the target's size and alignment
# (`[FFI-5]` checks a struct holding one against Clang), passed and returned as C passes the type,
# kept, copied and handed back, with no arithmetic of its own: C computes, and gives a `double`
# where Ember wants a number.

from std.ffi import c_longdouble
from std.mem import size_of
import c "./precise.c" with (overlay = "overlays/precise.em") as p

fn main():
    third: c_longdouble = p.precise_third()
    twice = p.precise_twice(third)
    println(f"{p.precise_to_double(twice):.4} {p.precise_to_double(third):.4}")
    m = p.precise_measure(3)
    kept = m
    println(kept.tag, p.precise_to_double(kept.value))
    println(size_of[c_longdouble]() >= 8)
