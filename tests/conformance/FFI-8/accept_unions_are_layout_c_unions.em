#$ test: run-pass
#$ rules: FFI-8, FFI-5
#$ profiles: debug, development, release
#$ stdout: 1065353216 1.0 4
#$ stdout: 10 20
#$ stdout: 2 6.5 3.25
#$ stdout: 6 3.0
# `[FFI-8]` — a C union is a `@layout(c, union)` struct: its fields share its storage (`[FFI-5]`
# checks its size and alignment against Clang's), it crosses by value and behind a pointer, it is
# made with one of its fields, and storing into a field is safe while reading one is `unsafe`:
# which field holds the value is the program's to know. An anonymous union member of a struct is
# read through, in `unsafe` too, and stored into; made, it names the member (`_anon0`).

from std.mem import size_of
import c "unions.h" with (overlay = "overlays/unions.em") as u

fn main():
    one = u.number_of_float(1.0)
    unsafe:  # SAFETY: C stored the float; its bits are read back as an integer, as C does
        println(one.i, one.f, size_of[u.Number]())
    n = u.Number(i = 5)
    n.i = 10
    before = n
    u.number_twice(n)
    unsafe:  # SAFETY: both hold `i`
        println(before.i, n.i)
    v = u.value_real(6.5)
    unsafe:  # SAFETY: kind 2 holds `real`
        println(v.kind, v.real, u.value_half(v))
    w = u.Value(kind = 1, _anon0 = u.Value_anon0(whole = 0))
    w.whole = 6
    unsafe:  # SAFETY: kind 1 holds `whole`
        println(w.whole, u.value_half(w))
