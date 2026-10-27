#$ test: run-pass
#$ rules: FFI-8, FFI-9, CG-LL-2
#$ profiles: debug, development, release
#$ stdout: 2.5 4.0 6.5
#$ stdout: 10 7
#$ stdout: 1.25 2.5
# `[FFI-8]` (ODR-249, ODR-270), `[FFI-9]` — a struct whose members are newtypes crosses to C and
# back by value as the struct of their representations does: `span_t`'s two `when_t`s are two
# doubles (two SSE registers on System V), `stamp_t`'s `when_t` and `tag_t` a double and an int (an
# SSE and an integer register), `lone_t`'s one `when_t` a double (an SSE register on System V, an
# integer one on Win64): the C ABI's classification reads each member's representation, under both
# backends (`[CG-LL-2]`).

from std.ffi import c_int

import c "./stamps.c" with (overlay = "overlays/stamps.em") as st

fn main():
    s = st.span_t(1.5 as st.when_t, 4.0 as st.when_t)
    shifted = st.span_shift(s, 2.5 as st.when_t)
    println(st.span_length(s), shifted.start as f64, shifted.end as f64)
    made = st.stamp_make(3.0 as st.when_t, 7 as st.tag_t)
    println(st.stamp_tag(made), made.tag as c_int)
    lone = st.lone_t(1.25 as st.when_t)
    println(st.lone_value(lone) as f64, st.lone_twice(lone).only as f64)
