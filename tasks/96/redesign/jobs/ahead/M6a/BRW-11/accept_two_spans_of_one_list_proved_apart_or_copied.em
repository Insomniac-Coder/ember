# Clause BRW-11.2: "are proved apart (`assert_disjoint`, two compares) or refused"
#$ test: run-pass
#$ rules: BRW-11, DSJ-5
#$ stdout: 3.0
#$ stdout: 5.0
# The two ways out the message gives: the proof before the call, and a copy of your own.

from std.mem import assert_disjoint

fn blend(ref mut dst: MutSpan[float], src: Span[float]):
    for i in 0..dst.len():
        dst[i] += src[i]

fn mix(ref mut tracks: Map[String, Array[float]], dst_name: StrView, src_name: StrView):
    d = ref mut tracks[dst_name].as_mut_span()
    s = tracks[src_name].as_span()
    assert_disjoint(d, s)
    blend(ref mut d, s)

fn mix_copy(ref mut tracks: Map[String, Array[float]], dst_name: StrView, src_name: StrView):
    old = copy of tracks[src_name]
    blend(ref mut tracks[dst_name].as_mut_span(), old.as_span())

fn main():
    tracks: Map[String, Array[float]] = Map()
    tracks.insert(owns String("a"), owns [1.0])
    tracks.insert(owns String("b"), owns [2.0])
    mix(ref mut tracks, "a", "b")
    println(tracks["a"][0])
    mix_copy(ref mut tracks, "a", "b")
    println(tracks["a"][0])
