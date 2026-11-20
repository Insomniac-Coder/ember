# Clause BRW-11.2: "This holds for spans of numbers too: two spans of one list given to one call,
# one of them changeable, are proved apart (`assert_disjoint`, two compares) or refused"
#$ test: compile-fail
#$ rules: BRW-11, DSJ-5
# The spec's message, with this file's line numbers.

fn blend(ref mut dst: MutSpan[float], src: Span[float]):
    for i in 0..dst.len():
        dst[i] += src[i]

fn mix(ref mut tracks: Map[String, Array[float]], dst_name: StrView, src_name: StrView):
    d = ref mut tracks[dst_name].as_mut_span()
    s = tracks[src_name].as_span()
    blend(ref mut d, s)    #$ error[E3021]: `blend` changes the elements of `d` (`tracks[dst_name]`, line 12) while `s` shows `tracks[src_name]` (line 13), and the two are one list when `dst_name` equals `src_name`; prove them apart first, `assert_disjoint(d, s)` before line 14, with `from std.mem import assert_disjoint` at the top, or blend from a copy: instead of lines 12 to 14 write `old = copy of tracks[src_name]` and `blend(ref mut tracks[dst_name].as_mut_span(), old.as_span())`

fn main():
    tracks: Map[String, Array[float]] = Map()
    tracks.insert(owns String("a"), owns [1.0])
    tracks.insert(owns String("b"), owns [2.0])
    mix(ref mut tracks, "a", "b")
