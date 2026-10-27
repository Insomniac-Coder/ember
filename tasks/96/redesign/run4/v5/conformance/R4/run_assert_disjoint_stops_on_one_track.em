#$ test: run-fail
#$ rules: FN-1
#$ panics: `assert_disjoint(d, s)` on line 18: the two spans overlap (`dst_name` and `src_name` name one track)
## design rules: R4
## R4, rules section 2: `assert_disjoint(d, s)` stops the program when the two spans overlap, here
## when `mix` is given one track's name twice, before `blend` runs with two names for one list
## (AN-14; the stop's text is rules section 2's).

from std.mem import assert_disjoint
from std.collections import Map

fn blend(ref mut dst: MutSpan[float], src: Span[float]):
    for i in 0..dst.len():
        dst[i] = dst[i] * 0.5 + src[i] * 0.5

fn mix(ref mut tracks: Map[String, Array[float]], dst_name: StrView, src_name: StrView):
    d = ref mut tracks[dst_name].as_mut_span()
    s = tracks[src_name].as_span()
    assert_disjoint(d, s)
    blend(ref mut d, s)

fn main():
    tracks = Map[String, Array[float]]()
    tracks[String("a")] = [1.0]
    mix(ref mut tracks, "a", "a")
    println(tracks["a"][0])
