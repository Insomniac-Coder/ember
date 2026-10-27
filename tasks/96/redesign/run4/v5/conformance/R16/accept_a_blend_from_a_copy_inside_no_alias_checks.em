#$ test: compile-pass
#$ rules: TIER-1, FN-1
## design rules: R16, R4
## R16 and R4: the lines E4030 gives: a blend from a list of its own is proved apart when compiling,
## so nothing runs that `@no_alias_checks` forbids (AN-14's way out).

fn blend(ref mut dst: MutSpan[float], src: Span[float]):
    for i in 0..dst.len():
        dst[i] += src[i]

@no_alias_checks
fn mix(ref mut tracks: Map[String, Array[float]], dst_name: StrView, src_name: StrView):
    old = copy of tracks[src_name]
    blend(ref mut tracks[dst_name].as_mut_span(), old.as_span())

fn main():
    tracks = {String("a"): [1.0], String("b"): [2.0]}
    mix(ref mut tracks, "a", "b")
