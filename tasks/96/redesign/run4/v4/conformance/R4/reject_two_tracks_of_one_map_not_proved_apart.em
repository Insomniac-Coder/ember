#$ test: compile-fail
#$ rules: R4
#$ help: assert_disjoint(d, s)
## R4: two spans the compiler cannot tell apart (two keys of one map) given to one call, one of them
## changeable, are refused until they are proved apart (AN-14's `mix` without its check, E3021).

from std.collections import Map

fn blend(ref mut dst: MutSpan[float], src: Span[float]):
    for i in 0..dst.len():
        dst[i] = dst[i] * 0.5 + src[i] * 0.5

fn mix(ref mut tracks: Map[String, Array[float]], dst_name: StrView, src_name: StrView):
    d = ref mut tracks[dst_name].as_mut_span()
    s = tracks[src_name].as_span()
    blend(ref mut d, s)    #$ error[E3021]: `blend` changes the elements of `d` (`tracks[dst_name]`, line 14) while `s` shows `tracks[src_name]` (line 15), and the two are one list when `dst_name` equals `src_name`; prove them apart first, `assert_disjoint(d, s)` before line 16, or blend from a copy: instead of lines 14 to 16 write `old = copy of tracks[src_name]` and `blend(ref mut tracks[dst_name].as_mut_span(), old.as_span())`

fn main():
    tracks = Map[String, Array[float]]()
    tracks[String("a")] = [1.0]
    tracks[String("b")] = [3.0]
    mix(ref mut tracks, "a", "b")
    println(tracks["a"][0])
