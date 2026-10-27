#$ test: compile-fail
#$ rules: R16, R4
## R16: `assert_disjoint` is the one run-time alias check left, and `@no_alias_checks` forbids it
## (E4030; AN-14's `mix`; the message is section 3's, verdict 98 removed R4's index compare). The program
## imports `assert_disjoint`, as every program that calls it does (step 3's re-check, wrong 3 to 6), so
## E4030 is its one fault.

from std.mem import assert_disjoint

fn blend(ref mut dst: MutSpan[float], src: Span[float]):
    for i in 0..dst.len():
        dst[i] += src[i]

@no_alias_checks
fn mix(ref mut tracks: Map[String, Array[float]], dst_name: StrView, src_name: StrView):
    d = ref mut tracks[dst_name].as_mut_span()
    s = tracks[src_name].as_span()
    assert_disjoint(d, s)    #$ error[E4030]: `mix` is `@no_alias_checks`, and `assert_disjoint(d, s)` (line 18) compares the two spans while running; blend from a copy, which needs no check: instead of lines 16 to 19 write `old = copy of tracks[src_name]` and `blend(ref mut tracks[dst_name].as_mut_span(), old.as_span())`, or remove `@no_alias_checks`
    blend(ref mut d, s)

fn main():
    tracks = {String("a"): [1.0], String("b"): [2.0]}
    mix(ref mut tracks, "a", "b")
