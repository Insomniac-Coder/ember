# Clause DIA-11.1: "For shape S1 the diagnostic names the `assert_disjoint` call and gives the lines
# that need no check while running (blending from a copy of your own), or removing
# `@no_alias_checks`; it never suggests restructuring code that cannot be improved."
#$ test: compile-fail
#$ rules: DIA-11, EFF-12, DSJ-5
#$ help: copy of
#$ help: @no_alias_checks
#$ not-help: restructur
# The spec gives no whole text for S1: the error claims the call's name, the helps the two ways
# out. Needs `assert_disjoint` (M6c).

from std.mem import assert_disjoint

fn blend(ref mut dst: MutSpan[float], src: Span[float]):
    for i in 0..dst.len():
        dst[i] += src[i]

@no_alias_checks
fn mix(ref mut tracks: Map[String, Array[float]], dst_name: StrView, src_name: StrView):
    d = ref mut tracks[dst_name].as_mut_span()
    s = tracks[src_name].as_span()
    assert_disjoint(d, s)    #$ error[E4030]: `assert_disjoint`
    blend(ref mut d, s)

fn main():
    tracks: Map[String, Array[float]] = Map()
    tracks.insert(owns String("a"), owns [1.0])
    tracks.insert(owns String("b"), owns [2.0])
    mix(ref mut tracks, "a", "b")
