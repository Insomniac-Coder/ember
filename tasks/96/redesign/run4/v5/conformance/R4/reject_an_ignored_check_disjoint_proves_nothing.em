#$ test: compile-fail
#$ rules: BRW-11
#$ help: write `assert_disjoint(d, s)` in place of line 18, with `from std.mem import assert_disjoint` at the top
## design rules: R4
## R4: a `check_disjoint` result bound and never looked at proves nothing, so the call stays refused
## as in `mix` without a check (AN-14, E3021). The way out imports `assert_disjoint`, which this program
## does not, so it compiles (step 3's re-check, wrong 3-6).

from std.mem import check_disjoint
from std.collections import Map

fn blend(ref mut dst: MutSpan[float], src: Span[float]):
    for i in 0..dst.len():
        dst[i] = dst[i] * 0.5 + src[i] * 0.5

fn mix(ref mut tracks: Map[String, Array[float]], dst_name: StrView, src_name: StrView):
    d = ref mut tracks[dst_name].as_mut_span()
    s = tracks[src_name].as_span()
    _checked = check_disjoint(d, s)
    blend(ref mut d, s)    #$ error[E3021]: `blend` changes the elements of `d` (`tracks[dst_name]`, line 16) while `s` shows `tracks[src_name]` (line 17), and the two are one list when `dst_name` equals `src_name`; prove them apart first, `assert_disjoint(d, s)` in place of line 18, with `from std.mem import assert_disjoint` at the top, or blend from a copy: instead of lines 16 to 19 write `old = copy of tracks[src_name]` and `blend(ref mut tracks[dst_name].as_mut_span(), old.as_span())`

fn main():
    tracks = Map[String, Array[float]]()
    tracks[String("a")] = [1.0]
    tracks[String("b")] = [3.0]
    mix(ref mut tracks, "a", "b")
    println(tracks["a"][0])
