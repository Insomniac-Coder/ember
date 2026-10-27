#$ test: compile-fail
#$ rules: R16, R4
## R16: `assert_disjoint` is the other run-time alias check `@no_alias_checks` forbids (E4030; AN-14's
## `mix`). The rules give the message for R4's index compare; this one is written to R18.

fn blend(ref mut dst: MutSpan[float], src: Span[float]):
    for i in 0..dst.len():
        dst[i] += src[i]

@no_alias_checks
fn mix(ref mut tracks: Map[String, Array[float]], dst_name: StrView, src_name: StrView):
    d = ref mut tracks[dst_name].as_mut_span()
    s = tracks[src_name].as_span()
    assert_disjoint(d, s)    #$ error[E4030]: `mix` is `@no_alias_checks`, and `assert_disjoint(d, s)` (line 14) compares the two spans while running; blend from a copy, which needs no check: instead of lines 12 to 15 write `old = copy of tracks[src_name]` and `blend(ref mut tracks[dst_name].as_mut_span(), old.as_span())`, or remove `@no_alias_checks`
    blend(ref mut d, s)

fn main():
    tracks = {String("a"): [1.0], String("b"): [2.0]}
    mix(ref mut tracks, "a", "b")
