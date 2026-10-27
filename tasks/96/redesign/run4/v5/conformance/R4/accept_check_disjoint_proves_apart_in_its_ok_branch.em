#$ test: run-pass
#$ rules: FN-1
#$ stdout: 2.0
#$ stdout: 1.0
## design rules: R4
## R4: `check_disjoint(d, s)` only reports; in its `Ok` branch the two are known apart and the call is
## accepted, in its `Err` branch nothing is blended and nothing stops (AN-14's `mix_any`).

from std.mem import check_disjoint
from std.collections import Map

fn blend(ref mut dst: MutSpan[float], src: Span[float]):
    for i in 0..dst.len():
        dst[i] = dst[i] * 0.5 + src[i] * 0.5

fn mix_any(ref mut tracks: Map[String, Array[float]], dst_name: StrView, src_name: StrView):
    d = ref mut tracks[dst_name].as_mut_span()
    s = tracks[src_name].as_span()
    match check_disjoint(d, s):
        Ok(_):
            blend(ref mut d, s)
        Err(_):
            pass

fn main():
    tracks = Map[String, Array[float]]()
    tracks[String("a")] = [1.0]
    tracks[String("b")] = [3.0]
    mix_any(ref mut tracks, "a", "b")
    println(tracks["a"][0])
    tracks[String("c")] = [1.0]
    mix_any(ref mut tracks, "c", "c")
    println(tracks["c"][0])
