# Clause BRW-12.7: "and before a `MutSpan`, or a tuple of them, a call gives it says the names bound
# from it may write the spans' elements (`d = ref mut tracks[dst_name].as_mut_span()`, `left, right =
# ref mut xs.split_at(mid)`, `[SPN-3]`)"
#$ test: run-pass
#$ rules: BRW-12, SPN-3, SPN-5
#$ stdout: 9 7 8 4
#$ stdout: true
# `s` is a `MutSpan`, which `split_at` splits into two (`[SPN-5]`; see Q6 for the spec's `xs`).

fn main():
    xs = [1, 2, 3, 4]
    d = ref mut xs.as_mut_span()
    d[0] = 9
    s = ref mut xs.as_mut_span()
    left, right = ref mut s.split_at(2)
    left[1] = 7
    right[0] = 8
    println(xs[0], xs[1], xs[2], xs[3])
    tracks: Map[String, Array[float]] = {String("a"): [1.0, 2.0]}
    dst_name = String("a")
    t = ref mut tracks[dst_name].as_mut_span()
    t[0] = 0.5
    println(tracks[dst_name][0] == 0.5)
