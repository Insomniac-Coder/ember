#$ test: run-pass
#$ rules: FN-1
#$ stdout: 2.0
## design rules: R4
## R4, rules section 2: `assert_disjoint(d, s)` compares the two address ranges once; afterwards the
## two names are known apart and the call is compiled with noalias (AN-14's `mix`).

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
    tracks[String("b")] = [3.0]
    mix(ref mut tracks, "a", "b")
    println(tracks["a"][0])
