# Clause EFF-12.1: "`@no_alias_checks` permits `RuntimeCheck(Bounds)`, `(Arithmetic)` and
# `(Stale)`; it forbids only `RuntimeCheck(Aliasing)`, the run-time compare of `assert_disjoint` or
# `check_disjoint` and the test of a `runtime` name (`[LT-48]`), because a data-oriented inner loop
# can always do without it"
#$ test: compile-fail
#$ rules: EFF-12, DSJ-6, DIA-11
# The spec's `mix` message, with this file's line numbers. Its `present` message needs a `runtime`
# parameter (`[LT-48]`) and a program the spec does not give: not here.

from std.mem import assert_disjoint

fn blend(ref mut dst: MutSpan[float], src: Span[float]):
    for i in 0..dst.len():
        dst[i] += src[i]

@no_alias_checks
fn mix(ref mut tracks: Map[String, Array[float]], dst_name: StrView, src_name: StrView):
    d = ref mut tracks[dst_name].as_mut_span()
    s = tracks[src_name].as_span()
    assert_disjoint(d, s)    #$ error[E4030]: `mix` is `@no_alias_checks`, and `assert_disjoint(d, s)` (line 20) compares the two spans while running; blend from a copy, which needs no check: instead of lines 18 to 21 write `old = copy of tracks[src_name]` and `blend(ref mut tracks[dst_name].as_mut_span(), old.as_span())`, or remove `@no_alias_checks`
    blend(ref mut d, s)

fn main():
    tracks: Map[String, Array[float]] = Map()
    tracks.insert(owns String("a"), owns [1.0])
    tracks.insert(owns String("b"), owns [2.0])
    mix(ref mut tracks, "a", "b")
