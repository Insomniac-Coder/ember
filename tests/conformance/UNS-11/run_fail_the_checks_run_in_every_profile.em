#$ test: run-fail
#$ rules: UNS-11, PRF-1
#$ profiles: debug, development, release
#$ panics: index 2 is out of bounds for a length of 2
# `[UNS-11]` — no profile turns a safety check off: the bounds check stops this program in every
# profile, `release` included.

fn pick(xs: Span[int], i: usize) -> int:
    return xs[i]

fn main():
    xs = [1, 2]
    println(pick(xs[..], xs.len()))
